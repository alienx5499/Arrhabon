//! Registry route: the active universities and attestors a student can pick.

use serde_json::json;

use super::routes::assert_error;
use super::*;

#[tokio::test]
async fn registry_lists_active_parties() {
    let s = state(true, None);
    let (status, body) = get(&s, "/registry").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({
            "universities": [{ "address": b58(UNIVERSITY) }],
            "attestors": [{ "address": b58(ATTESTOR) }],
        })
    );
}

#[tokio::test]
async fn registry_without_chain_is_unavailable() {
    let s = state(false, None);
    let res = get(&s, "/registry").await;
    assert_error(
        &res,
        StatusCode::SERVICE_UNAVAILABLE,
        "chain_not_initialised",
    );
}
