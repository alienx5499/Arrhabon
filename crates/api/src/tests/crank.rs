//! Crank tick: one failing item never stops the others.

use arrhabon_app::commands::EXPIRY_GRACE_SECS;
use arrhabon_domain::VaultStatus;

use super::*;
use std::time::Duration;

use crate::crank::{MAX_BACKOFF, next_delay, tick};

#[tokio::test]
async fn tick_expires_and_refunds_around_partial_failures() {
    let s = state(true, None);
    let l = ledger(&s);
    let due = NOW - EXPIRY_GRACE_SECS;
    let broken = seed_vault(l, 1, due, &[(AUNT, 100)]);
    let partial = seed_vault(l, 2, due, &[(AUNT, 300), (UNCLE, 200)]);
    let fine = seed_vault(l, 3, due, &[(AUNT, 100)]);
    let open = seed_vault(l, 4, NOW + 120, &[(AUNT, 100)]);
    l.fail_for(broken);
    l.fail_for(UNCLE);

    assert!(
        tick(l, &s.clock).await,
        "injected failures are outages: back off"
    );

    let status = |id| l.stored_vault(id).unwrap().status();
    assert_eq!(status(broken), VaultStatus::Open); // expire failed, retried next tick
    assert_eq!(status(partial), VaultStatus::Refunding); // UNCLE's refund failed
    assert_eq!(status(fine), VaultStatus::Closed);
    assert_eq!(status(open), VaultStatus::Open);
}

#[tokio::test]
async fn crank_task_survives_failing_ticks() {
    let s = state(true, None);
    let broken = seed_vault(ledger(&s), 1, NOW - EXPIRY_GRACE_SECS, &[(AUNT, 100)]);
    ledger(&s).fail_for(broken);
    let task = crate::crank::spawn(s.clone(), std::time::Duration::from_millis(5));
    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    assert!(!task.is_finished());
    task.abort();
}

#[test]
fn backoff_doubles_on_outages_caps_and_resets() {
    let base = Duration::from_secs(10);
    let secs = Duration::from_secs;
    assert_eq!(next_delay(base, base, true), secs(20));
    assert_eq!(next_delay(base, secs(80), true), MAX_BACKOFF);
    assert_eq!(next_delay(base, MAX_BACKOFF, true), MAX_BACKOFF);
    assert_eq!(next_delay(base, MAX_BACKOFF, false), base);
    let slow = secs(300); // a base above the cap is never shortened
    assert_eq!(next_delay(slow, slow, true), slow);
}
