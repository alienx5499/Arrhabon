//! Cross-chain funding routes: quote and swap status.

use arrhabon_app::ports::SwapStatus;
use serde_json::json;

use super::routes::assert_error;
use super::*;

fn quote_body(vault: AccountId) -> Value {
    json!({
        "vault": b58(vault), "origin_chain": "base", "origin_asset": "usdc",
        "origin_amount": "5000000", "funder": b58(AUNT), "refund_to": "0xabc", "slippage_bps": 100,
    })
}

#[tokio::test]
async fn funding_quote_and_status() {
    let s = state(true, None);
    let open = seed_vault(ledger(&s), 1, NOW + 120, &[]);
    let (status, quote) = post(&s, "/funding/quote", quote_body(open)).await;
    assert_eq!(status, StatusCode::OK, "{quote}");
    assert_eq!(quote["min_usdc_out"], 4_000_000);
    assert!(quote.get("expected_usdc_out").is_none());
    let addr = quote["deposit_address"].as_str().unwrap().to_owned();
    s.quoter.set_status(&addr, SwapStatus::Success);
    let (status, body) = get(&s, &format!("/funding/status?deposit_address={addr}")).await;
    assert_eq!(
        (status, &body["status"]),
        (StatusCode::OK, &json!("success"))
    );

    let unknown = get(&s, "/funding/status?deposit_address=nope").await;
    assert_error(&unknown, StatusCode::BAD_REQUEST, "rejected");
    let missing = get(&s, "/funding/status").await;
    assert_error(&missing, StatusCode::BAD_REQUEST, "invalid_input");
}

#[tokio::test]
async fn funding_quote_errors() {
    let s = state(true, None);
    let late = seed_vault(ledger(&s), 1, NOW - 1, &[]);
    let closed = post(&s, "/funding/quote", quote_body(late)).await;
    assert_error(&closed, StatusCode::CONFLICT, "DeadlinePassed");
    let mut body = quote_body(late);
    body["origin_amount"] = json!("1e18");
    let bad = post(&s, "/funding/quote", body).await;
    assert_error(&bad, StatusCode::BAD_REQUEST, "invalid_input");
}

#[tokio::test]
async fn funding_quote_caps_free_text_fields() {
    let s = state(true, None);
    let open = seed_vault(ledger(&s), 1, NOW + 120, &[]);
    for field in ["origin_chain", "origin_asset", "refund_to"] {
        let mut body = quote_body(open);
        body[field] = "x".repeat(crate::routes::MAX_FIELD_LEN + 1).into();
        let res = post(&s, "/funding/quote", body).await;
        assert_error(&res, StatusCode::BAD_REQUEST, "invalid_input");
    }
}
