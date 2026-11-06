//! Credentials stay out of URLs in errors, error bodies, and Debug output.

mod common;

use std::time::Duration;

use arrhabon_app::PortError;
use arrhabon_app::ports::{SwapQuoter, SwapStatus};
use arrhabon_aurora_adapter::{Api, AuroraConfig, AuroraQuoter};
use common::{request, tokens};
use serde_json::json;
use wiremock::matchers::{header, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn aurora_key_goes_in_the_path_and_never_in_errors() {
    let server = MockServer::start().await;
    Mock::given(path("/api/status/secret-key"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&server)
        .await;
    Mock::given(path("/api/tokens/secret-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"asset_stats": [], "tokens": tokens()})),
        )
        .mount(&server)
        .await;
    let mut config = AuroraConfig::dry(server.uri());
    config.api = Api::Aurora {
        api_key: "secret-key".into(),
    };
    config.timeout = Duration::from_millis(300);
    assert!(!format!("{config:?}").contains("secret-key"));
    let q = AuroraQuoter::new(config).unwrap();
    let err = q.status("x").await.unwrap_err();
    assert!(!err.to_string().contains("secret-key"), "{err}");
    // Aurora's wrapped token list parses; the quote then 404s on the unmocked path.
    assert!(matches!(
        q.quote(request()).await,
        Err(PortError::Rejected { .. })
    ));
}

#[tokio::test]
async fn error_bodies_echoing_the_key_are_redacted_and_truncated() {
    let server = MockServer::start().await;
    let echo = format!("no route /api/status/secret-key {}", "x".repeat(500));
    Mock::given(path("/api/status/secret-key"))
        .respond_with(ResponseTemplate::new(404).set_body_string(echo))
        .mount(&server)
        .await;
    let mut config = AuroraConfig::dry(server.uri());
    config.api = Api::Aurora {
        api_key: "secret-key".into(),
    };
    let err = AuroraQuoter::new(config)
        .unwrap()
        .status("x")
        .await
        .unwrap_err();
    let PortError::Rejected { message, .. } = err else {
        panic!("expected Rejected, got {err:?}");
    };
    assert!(!message.contains("secret-key"), "{message}");
    assert!(message.contains("/api/status/<redacted>"), "{message}");
    assert!(message.len() < 250, "{} chars", message.len());
}

#[tokio::test]
async fn jwt_is_sent_as_bearer() {
    let server = MockServer::start().await;
    Mock::given(path("/v0/status"))
        .and(header("authorization", "Bearer jwt-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "SUCCESS"})))
        .expect(1)
        .mount(&server)
        .await;
    let mut config = AuroraConfig::dry(server.uri());
    config.api = Api::OneClick {
        jwt: Some("jwt-1".into()),
    };
    assert!(!format!("{config:?}").contains("jwt-1"));
    let q = AuroraQuoter::new(config).unwrap();
    assert_eq!(q.status("a").await.unwrap(), SwapStatus::Success);
}
