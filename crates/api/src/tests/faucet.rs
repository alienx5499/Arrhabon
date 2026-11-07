//! DEMO faucet route: grant, shortfall-only, cooldown, disabled.

use arrhabon_app::commands::{LAMPORTS_GRANT, USDC_GRANT};
use arrhabon_app::fakes::FakeFaucet;
use serde_json::json;

use super::routes::assert_error;
use super::*;

const JUDGE: AccountId = AccountId([9; 32]);

#[tokio::test]
async fn an_empty_wallet_gets_sol_and_usdc() {
    let faucet = FakeFaucet::default();
    let s = faucet_state(faucet.clone());
    let (status, body) = post(&s, "/demo/faucet", json!({ "wallet": b58(JUDGE) })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["lamports"], LAMPORTS_GRANT);
    assert_eq!(body["usdc_micro"], USDC_GRANT.micro().to_string());
    assert_eq!(body["demo"], true);
    assert_eq!(faucet.sent(), vec![(JUDGE, LAMPORTS_GRANT, USDC_GRANT)]);
}

#[tokio::test]
async fn a_second_claim_inside_the_cooldown_is_refused() {
    let s = faucet_state(FakeFaucet::default());
    let body = json!({ "wallet": b58(JUDGE) });
    assert_eq!(
        post(&s, "/demo/faucet", body.clone()).await.0,
        StatusCode::OK
    );
    let res = post(&s, "/demo/faucet", body).await;
    assert_error(&res, StatusCode::TOO_MANY_REQUESTS, "faucet_cooldown");
}

#[tokio::test]
async fn a_funded_wallet_gets_409_and_can_retry_later() {
    let faucet = FakeFaucet::default();
    faucet.set(JUDGE, LAMPORTS_GRANT, USDC_GRANT);
    let s = faucet_state(faucet.clone());
    let res = post(&s, "/demo/faucet", json!({ "wallet": b58(JUDGE) })).await;
    assert_error(&res, StatusCode::CONFLICT, "already_funded");
    assert_eq!(
        s.faucet_claims.lock().unwrap().grants_in_window(),
        0,
        "a refused claim uses no budget"
    );
    assert!(faucet.sent().is_empty());
}

#[tokio::test]
async fn the_faucet_is_off_unless_enabled() {
    let s = state(true, None);
    let res = post(&s, "/demo/faucet", json!({ "wallet": b58(JUDGE) })).await;
    assert_error(&res, StatusCode::FORBIDDEN, "faucet_disabled");
}

#[tokio::test]
async fn a_bad_wallet_is_400() {
    let s = faucet_state(FakeFaucet::default());
    let (status, _) = post(&s, "/demo/faucet", json!({ "wallet": "not-a-key" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_hourly_budget_stops_fresh_wallet_draining() {
    use crate::faucet::GRANTS_PER_HOUR;
    let s = faucet_state(FakeFaucet::default());
    for i in 0..GRANTS_PER_HOUR {
        let w = AccountId([10 + i as u8; 32]);
        assert_eq!(
            post(&s, "/demo/faucet", json!({ "wallet": b58(w) }))
                .await
                .0,
            StatusCode::OK
        );
    }
    let res = post(
        &s,
        "/demo/faucet",
        json!({ "wallet": b58(AccountId([200; 32])) }),
    )
    .await;
    assert_error(&res, StatusCode::TOO_MANY_REQUESTS, "faucet_busy");
}

#[test]
fn every_grant_counts_against_the_budget_even_from_one_wallet() {
    use crate::faucet::{CLAIM_COOLDOWN_SECS, Claims};
    let mut c = Claims::default();
    c.reserve(JUDGE, 0).unwrap();
    c.reserve(JUDGE, CLAIM_COOLDOWN_SECS).unwrap();
    assert_eq!(
        c.grants_in_window(),
        2,
        "a returning wallet uses a second slot"
    );
}

#[test]
fn a_failed_send_releases_only_its_own_slot() {
    use crate::faucet::{CLAIM_COOLDOWN_SECS, Claims};
    let mut c = Claims::default();
    c.reserve(JUDGE, 0).unwrap();
    let slot = c.reserve(JUDGE, CLAIM_COOLDOWN_SECS).unwrap();
    c.release(slot);
    assert_eq!(c.grants_in_window(), 1, "the first grant stays counted");
    assert!(
        c.reserve(JUDGE, CLAIM_COOLDOWN_SECS + 1).is_ok(),
        "the earlier claim time is restored, so its cooldown has passed"
    );
}
