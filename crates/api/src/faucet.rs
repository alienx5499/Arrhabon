//! DEMO ONLY: `POST /demo/faucet` tops a wallet up with devnet SOL and test USDC.

use arrhabon_app::commands::ClaimTestFunds;
use arrhabon_app::ports::Clock;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

use arrhabon_domain::AccountId;

use crate::dto;
use crate::error::{ApiError, Body};
use crate::routes::Res;
use crate::state::{AppState, Ports};

/// One successful claim per wallet per window; the use case also sends only the shortfall.
pub const CLAIM_COOLDOWN_SECS: i64 = 600;
/// Fresh wallets bypass the per-wallet cooldown, so all grants share an hourly budget:
/// at most 1.5 SOL an hour can leave the faucet, keeping it alive for judges.
pub const GRANTS_PER_HOUR: usize = 30;
const HOUR_SECS: i64 = 3600;

#[derive(Deserialize)]
pub struct FaucetBody {
    wallet: String,
}

pub async fn demo_faucet<P: Ports>(State(s): State<AppState<P>>, Body(b): Body<FaucetBody>) -> Res {
    if !s.faucet_on {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "faucet_disabled",
            "DEMO faucet is disabled; set FAUCET=true to enable it",
        ));
    }
    let faucet = s.faucet.get().ok_or_else(ApiError::chain_not_initialised)?;
    let wallet = dto::account(&b.wallet)?;
    let now = s.clock.now();
    let slot = s
        .faucet_claims
        .lock()
        .expect("faucet claims lock poisoned")
        .reserve(wallet, now)?;
    let claim = ClaimTestFunds { wallet };
    let grant = match claim.execute(faucet).await {
        Ok(g) => g,
        Err(e) => {
            s.faucet_claims
                .lock()
                .expect("faucet claims lock poisoned")
                .release(slot);
            return Err(e.into());
        }
    };
    tracing::info!(wallet = %b.wallet, lamports = grant.lamports, usdc = grant.usdc.micro(), "DEMO faucet grant");
    Ok(Json(json!({
        "demo": true,
        "notice": "DEMO faucet: devnet SOL and test USDC only, worth nothing",
        "signature": grant.signature.0,
        "lamports": grant.lamports,
        "usdc_micro": grant.usdc.micro().to_string(),
    })))
}

/// Per-wallet cooldowns plus a log of every grant in the last hour (the global budget).
#[derive(Debug, Default)]
pub struct Claims {
    last_by_wallet: HashMap<AccountId, i64>,
    grants: VecDeque<i64>,
}

/// A held budget slot; give it back with `Claims::release` if the send fails.
#[derive(Debug)]
pub struct Slot {
    wallet: AccountId,
    at: i64,
    previous: Option<i64>,
}

impl Claims {
    /// Refuses when the hourly budget is spent or the wallet is in its cooldown; otherwise
    /// holds one budget slot and starts the wallet's cooldown.
    pub fn reserve(&mut self, wallet: AccountId, now: i64) -> Result<Slot, ApiError> {
        while self.grants.front().is_some_and(|t| now - t >= HOUR_SECS) {
            self.grants.pop_front();
        }
        if self.grants.len() >= GRANTS_PER_HOUR {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "faucet_busy",
                "the test faucet hit its hourly limit; try again later or use faucet.solana.com",
            ));
        }
        if self
            .last_by_wallet
            .get(&wallet)
            .is_some_and(|t| now - t < CLAIM_COOLDOWN_SECS)
        {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "faucet_cooldown",
                "this wallet was just funded; try again in a few minutes",
            ));
        }
        let previous = self.last_by_wallet.insert(wallet, now);
        self.grants.push_back(now);
        Ok(Slot {
            wallet,
            at: now,
            previous,
        })
    }

    /// Undoes one reservation: frees its budget slot and restores the wallet's last claim.
    pub fn release(&mut self, slot: Slot) {
        if let Some(i) = self.grants.iter().rposition(|t| *t == slot.at) {
            self.grants.remove(i);
        }
        match slot.previous {
            Some(t) => self.last_by_wallet.insert(slot.wallet, t),
            None => self.last_by_wallet.remove(&slot.wallet),
        };
    }

    #[cfg(test)]
    pub fn grants_in_window(&self) -> usize {
        self.grants.len()
    }
}
