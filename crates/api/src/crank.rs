//! Refund crank: on an interval, expire due vaults then refund what is owed; never panics out.

use std::time::Duration;

use arrhabon_app::AppError;
use arrhabon_app::address::to_base58;
use arrhabon_app::commands::{Done, ExpireDue, RunRefunds};
use arrhabon_app::ports::{Clock, VaultLedger};

use crate::error::scrub;
use crate::state::{AppState, Ports};

/// Longest wait between crank runs while the RPC keeps failing.
pub const MAX_BACKOFF: Duration = Duration::from_secs(120);

/// Wait before the next run: back to `base` after a clean run, doubled (capped at
/// `MAX_BACKOFF`) after one that hit an unavailable or rate-limited RPC.
pub fn next_delay(base: Duration, previous: Duration, unavailable: bool) -> Duration {
    match unavailable {
        false => base,
        true => (previous * 2).clamp(base, MAX_BACKOFF.max(base)),
    }
}

/// One crank run; every per-item failure is logged and the rest still run.
/// Returns whether any call found the RPC unavailable.
pub async fn tick(ledger: &impl VaultLedger, clock: &impl Clock) -> bool {
    let mut unavailable = false;
    let mut note = |e: &AppError| unavailable |= matches!(e, AppError::Unavailable(_));
    match ExpireDue.execute(ledger, clock).await {
        Ok(outcomes) => {
            for o in outcomes {
                let vault = to_base58(o.item);
                match o.result {
                    Ok(done) => tracing::info!(%vault, result = %describe(&done), "crank expire"),
                    Err(e) => {
                        note(&e);
                        tracing::warn!(%vault, error = %scrub(&e.to_string()), "crank expire failed")
                    }
                }
            }
        }
        Err(e) => {
            note(&e);
            tracing::warn!(error = %scrub(&e.to_string()), "crank: listing due vaults failed")
        }
    }
    match RunRefunds.execute(ledger).await {
        Ok(outcomes) => {
            for o in outcomes {
                let (vault, funder) = (to_base58(o.item.vault), to_base58(o.item.funder));
                match o.result {
                    Ok(done) => {
                        tracing::info!(%vault, %funder, result = %describe(&done), "crank refund")
                    }
                    Err(e) => {
                        note(&e);
                        tracing::warn!(%vault, %funder, error = %scrub(&e.to_string()), "crank refund failed")
                    }
                }
            }
        }
        Err(e) => {
            note(&e);
            tracing::warn!(error = %scrub(&e.to_string()), "crank: listing pending refunds failed")
        }
    }
    unavailable
}

fn describe(done: &Done) -> String {
    match done {
        Done::Submitted(sig) => format!("submitted {}", sig.0),
        Done::AlreadyApplied => "already applied".into(),
        Done::LeftForFunder => "left for the funder to claim (no token account)".into(),
    }
}

/// Runs `tick` every `every` once the chain is ready, backing off while the RPC is
/// unavailable. Each tick runs in its own task, so a panic in one is logged and the next
/// tick still runs.
pub fn spawn<P: Ports>(state: AppState<P>, every: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut delay = every;
        loop {
            tokio::time::sleep(delay).await;
            if state.ledger.get().is_none() {
                continue; // chain not initialised yet
            }
            let s = state.clone();
            let run = tokio::spawn(async move {
                let ledger = s.ledger.get().expect("checked above");
                P::refresh_clock(ledger, &s.clock).await;
                tick(ledger, &s.clock).await
            });
            let unavailable = run.await.unwrap_or_else(|_| {
                tracing::error!("crank tick panicked; continuing");
                false
            });
            delay = next_delay(every, delay, unavailable);
            if unavailable {
                tracing::warn!("crank: RPC unavailable; next run in {}s", delay.as_secs());
            }
        }
    })
}
