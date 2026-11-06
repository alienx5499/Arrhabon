//! Shared 1Click mock fixtures.
#![allow(dead_code)]

use std::time::Duration;

use arrhabon_app::ports::{DestinationAsset, QuoteRequest};
use arrhabon_aurora_adapter::{AuroraConfig, AuroraQuoter};
use arrhabon_domain::AccountId;
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const BASE_USDC: &str = "nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near";
pub const SOL_USDC: &str = "nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near";

pub fn tokens() -> Value {
    json!([
        {"assetId": BASE_USDC, "decimals": 6, "blockchain": "base", "symbol": "USDC", "price": 1.0,
         "priceUpdatedAt": "2026-10-25T07:16:00.395Z", "contractAddress": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"},
        {"assetId": SOL_USDC, "decimals": 6, "blockchain": "sol", "symbol": "USDC", "price": 1.0,
         "priceUpdatedAt": "2026-10-25T07:16:00.395Z", "contractAddress": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}
    ])
}

pub fn request() -> QuoteRequest {
    QuoteRequest {
        origin_chain: "base".into(),
        origin_asset: "USDC".into(),
        origin_amount: 5_000_000,
        recipient: AccountId([7; 32]),
        refund_to: "0x1111111111111111111111111111111111111111".into(),
        slippage_bps: 100,
        destination: DestinationAsset::SolanaUsdc,
    }
}

pub fn dry_quote_response() -> Value {
    json!({"quote": {"amountIn": "5000000", "amountInUsd": "4.997550000000", "amountOut": "4989685",
        "amountOutUsd": "4.987240054350", "minAmountOut": "4939788", "timeEstimate": 32},
        "quoteRequest": {}, "signature": "ed25519:x", "timestamp": "2026-10-25T07:16:50.355Z"})
}

pub async fn server_with_tokens() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tokens()))
        .expect(1)
        .mount(&server)
        .await;
    server
}

pub fn quoter(server: &MockServer, live: bool) -> AuroraQuoter {
    let mut config = AuroraConfig::dry(server.uri()).with_live(live);
    config.timeout = Duration::from_millis(500);
    AuroraQuoter::new(config).unwrap()
}
