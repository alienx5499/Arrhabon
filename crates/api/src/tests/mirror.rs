use arrhabon_app::fakes::FakeFaucet;
use arrhabon_app::ports::SwapStatus;
use arrhabon_domain::Usdc;
use axum::http::StatusCode;
use serde_json::json;

use super::*;
use crate::mirror::MAX_REMEMBERED_QUOTES;

#[tokio::test]
async fn unknown_deposit_404() {
    let s = mirror_state(FakeFaucet::default(), true);
    let (status, body) = post(
        &s,
        "/funding/mirror",
        json!({ "deposit_address": "unknown-deposit" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "unknown_deposit");
}

#[tokio::test]
async fn disabled_403() {
    // Both FAUCET=true and AURORA_LIVE=true are required.
    let s = state(true, None);
    let (status, body) = post(
        &s,
        "/funding/mirror",
        json!({ "deposit_address": "deposit-1" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "mirror_disabled");
}

#[tokio::test]
async fn success_200_then_409_already_mirrored_with_same_signature() {
    let faucet = FakeFaucet::default();
    let s = mirror_state(faucet.clone(), true);
    let v = seed_vault(ledger(&s), 1, NOW + 600, &[]);

    let (q_status, quote) = post(
        &s,
        "/funding/quote",
        json!({
            "vault": b58(v),
            "origin_chain": "base",
            "origin_asset": "usdc",
            "origin_amount": "50000000",
            "funder": b58(UNCLE),
            "refund_to": "0xuncle",
            "slippage_bps": 50,
        }),
    )
    .await;
    assert_eq!(q_status, StatusCode::OK);
    let deposit = quote["deposit_address"].as_str().unwrap();

    s.quoter.set_status(deposit, SwapStatus::Success);

    let (status1, body1) = post(&s, "/funding/mirror", json!({ "deposit_address": deposit })).await;
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(body1["demo"], true);
    assert_eq!(body1["funder"], b58(UNCLE));
    assert_eq!(body1["vault"], b58(v));
    let sig1 = body1["signature"].as_str().unwrap();
    assert_eq!(body1["usdc_micro"], "49000000");

    let (status2, body2) = post(&s, "/funding/mirror", json!({ "deposit_address": deposit })).await;
    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(body2["code"], "already_mirrored");
    assert_eq!(body2["signature"], sig1);
}

#[tokio::test]
async fn funder_comes_from_quote_not_body() {
    let faucet = FakeFaucet::default();
    let s = mirror_state(faucet.clone(), true);
    let v = seed_vault(ledger(&s), 1, NOW + 600, &[]);

    let (_, quote) = post(
        &s,
        "/funding/quote",
        json!({
            "vault": b58(v),
            "origin_chain": "base",
            "origin_asset": "usdc",
            "origin_amount": "50000000",
            "funder": b58(AUNT),
            "refund_to": "0xaunt",
            "slippage_bps": 50,
        }),
    )
    .await;
    let deposit = quote["deposit_address"].as_str().unwrap();
    s.quoter.set_status(deposit, SwapStatus::Success);

    // Mirror body accepts only deposit_address; funder cannot be spoofed.
    let (status, body) = post(&s, "/funding/mirror", json!({ "deposit_address": deposit })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["funder"], b58(AUNT));
    assert_eq!(faucet.sent()[0].0, AUNT);
}

#[tokio::test]
async fn bounded_memory_drops_oldest() {
    let s = mirror_state(FakeFaucet::default(), true);
    let mut mirror = s.mirror.lock().unwrap();

    for i in 0..=MAX_REMEMBERED_QUOTES {
        mirror.remember(
            format!("deposit-{i}"),
            UNCLE,
            STUDENT,
            Usdc::new(10_000_000),
        );
    }

    // The oldest entry (deposit-0) was dropped to keep bounds at MAX_REMEMBERED_QUOTES.
    let err = mirror.reserve("deposit-0").unwrap_err();
    assert_eq!(err.code, "unknown_deposit");

    // The next entry and the newest entry are still present.
    assert!(mirror.reserve("deposit-1").is_ok());
    assert!(
        mirror
            .reserve(&format!("deposit-{MAX_REMEMBERED_QUOTES}"))
            .is_ok()
    );
}
