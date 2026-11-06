//! AuroraQuoter against a local mock of the 1Click API; one ignored live dry quote.

mod common;

use std::time::Duration;

use arrhabon_app::PortError;
use arrhabon_app::ports::{QuoteRequest, SwapQuoter, SwapStatus};
use arrhabon_aurora_adapter::{AuroraConfig, AuroraQuoter};
use arrhabon_domain::{AccountId, Usdc};
use common::*;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn dry_quote_maps_the_request_and_response() {
    let server = server_with_tokens().await;
    let recipient = bs58_of([7; 32]);
    Mock::given(method("POST"))
        .and(path("/v0/quote"))
        .and(body_partial_json(json!({
            "dry": true, "swapType": "EXACT_INPUT", "slippageTolerance": 100,
            "originAsset": BASE_USDC, "depositType": "ORIGIN_CHAIN", "destinationAsset": SOL_USDC,
            "amount": "5000000", "refundTo": "0x1111111111111111111111111111111111111111",
            "refundType": "ORIGIN_CHAIN", "recipient": recipient, "recipientType": "DESTINATION_CHAIN"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(dry_quote_response()))
        .expect(2)
        .mount(&server)
        .await;
    let q = quoter(&server, false);
    let quote = q.quote(request()).await.unwrap();
    assert_eq!(quote.deposit_address, "");
    assert_eq!(
        quote.min_usdc_out,
        Usdc::new(4_939_788),
        "guaranteed floor: minAmountOut, not amountOut"
    );
    assert_eq!(quote.fee, Usdc::new(10_310));
    assert!(quote.expires_at > 1_700_000_000);
    // Second quote reuses the cached token list (tokens mock expects exactly one call).
    q.quote_dry(request()).await.unwrap();
}

#[tokio::test]
async fn live_quote_is_refused_without_the_flag_and_sends_nothing() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let err = quoter(&server, false)
        .quote_live(request())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PortError::Rejected { ref message, .. } if message.contains("AURORA_LIVE"))
    );
}

#[tokio::test]
async fn live_quote_with_the_flag_returns_the_deposit_address() {
    let server = server_with_tokens().await;
    let mut body = dry_quote_response();
    body["quote"]["depositAddress"] = json!("0xdeposit");
    body["quote"]["timeWhenInactive"] = json!("2026-01-01T00:00:00.000Z");
    Mock::given(method("POST"))
        .and(path("/v0/quote"))
        .and(body_partial_json(json!({"dry": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let quote = quoter(&server, true).quote(request()).await.unwrap();
    assert_eq!(quote.deposit_address, "0xdeposit");
    assert_eq!(
        quote.expires_at, 1_767_225_600,
        "earlier of timeWhenInactive and our deadline"
    );
}

#[tokio::test]
async fn unsupported_origin_is_rejected() {
    let server = server_with_tokens().await;
    let req = QuoteRequest {
        origin_chain: "tron".into(),
        ..request()
    };
    let err = quoter(&server, false).quote(req).await.unwrap_err();
    assert!(
        matches!(err, PortError::Rejected { ref message, .. } if message.contains("not supported"))
    );
}

#[tokio::test]
async fn statuses_map_and_keep_the_raw_value() {
    let server = MockServer::start().await;
    for (addr, raw) in [
        ("a", "SUCCESS"),
        ("b", "REFUNDED"),
        ("c", "FAILED"),
        ("d", "PROCESSING"),
        ("e", "SOMETHING_NEW"),
    ] {
        Mock::given(method("GET"))
            .and(path("/v0/status"))
            .and(query_param("depositAddress", addr))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"status": raw, "updatedAt": "x"})),
            )
            .mount(&server)
            .await;
    }
    let q = quoter(&server, false);
    assert_eq!(q.status("a").await.unwrap(), SwapStatus::Success);
    assert_eq!(q.status("b").await.unwrap(), SwapStatus::Refunded);
    assert_eq!(q.status("c").await.unwrap(), SwapStatus::Failed);
    assert_eq!(q.status("d").await.unwrap(), SwapStatus::Pending);
    let unknown = q.swap_state("e").await.unwrap();
    assert_eq!(
        (unknown.status, unknown.raw.as_str()),
        (SwapStatus::Pending, "SOMETHING_NEW")
    );
    assert!(matches!(
        q.status("").await,
        Err(PortError::Rejected { .. })
    ));
}

#[tokio::test]
async fn http_errors_map_to_port_errors() {
    let server = MockServer::start().await;
    let cases = [("bad", 400), ("missing", 404), ("busy", 429), ("down", 503)];
    for (addr, code) in cases {
        Mock::given(path("/v0/status"))
            .and(query_param("depositAddress", addr))
            .respond_with(
                ResponseTemplate::new(code).set_body_json(json!({"message": format!("m{code}")})),
            )
            .mount(&server)
            .await;
    }
    Mock::given(path("/v0/status"))
        .and(query_param("depositAddress", "slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&server)
        .await;
    Mock::given(path("/v0/status"))
        .and(query_param("depositAddress", "garbage"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let q = quoter(&server, false);
    assert_eq!(
        q.status("bad").await,
        Err(PortError::rejected("aurora 400: m400"))
    );
    assert_eq!(
        q.status("missing").await,
        Err(PortError::rejected("aurora 404: m404"))
    );
    assert_eq!(
        q.status("busy").await,
        Err(PortError::Unavailable("aurora 429: m429".into()))
    );
    assert_eq!(
        q.status("down").await,
        Err(PortError::Unavailable("aurora 503: m503".into()))
    );
    assert!(
        matches!(q.status("slow").await, Err(PortError::Unavailable(m)) if m.contains("timeout"))
    );
    assert!(matches!(
        q.status("garbage").await,
        Err(PortError::Rejected { .. })
    ));
}

#[tokio::test]
async fn token_decimals_come_from_the_token_list() {
    let server = server_with_tokens().await;
    let q = quoter(&server, false);
    assert_eq!(q.token_decimals("base", "USDC").await.unwrap(), 6);
    assert_eq!(q.token_decimals("sol", SOL_USDC).await.unwrap(), 6);
    assert!(matches!(
        q.token_decimals("tron", "USDC").await,
        Err(PortError::Rejected { .. })
    ));
}

/// Needs network; dry only, no deposit address is reserved and nothing moves.
#[tokio::test]
#[ignore = "calls the public 1Click API"]
async fn live_dry_quote_base_usdc_to_solana_usdc() {
    let q = AuroraQuoter::new(AuroraConfig::from_env()).unwrap();
    let quote = q.quote_dry(request()).await.unwrap();
    println!("{quote:?}");
    assert_eq!(quote.deposit_address, "");
    assert!(quote.min_usdc_out > Usdc::new(4_000_000));
}

fn bs58_of(bytes: [u8; 32]) -> String {
    arrhabon_app::address::to_base58(AccountId(bytes))
}
