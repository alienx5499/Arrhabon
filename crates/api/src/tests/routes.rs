//! Every route on the fakes, including its error statuses and the `{error, code}` body.

use arrhabon_app::AppError;
use arrhabon_app::fakes::sign_as;
use arrhabon_app::ports::UnsignedTx;
use arrhabon_domain::VaultStatus;
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::json;

use super::*;
use crate::error::ApiError;

pub fn assert_error(res: &(StatusCode, Value), status: StatusCode, code: &str) {
    assert_eq!(res.0, status, "{}", res.1);
    assert_eq!(res.1["code"], code, "{}", res.1);
    assert!(res.1["error"].is_string());
}

/// What the user's wallet does: sign the base64 unsigned tx and return base64.
fn wallet_sign(unsigned_b64: &Value, signer: AccountId) -> String {
    let bytes = B64.decode(unsigned_b64.as_str().unwrap()).unwrap();
    let tx = UnsignedTx {
        bytes,
        last_valid_block_height: 0,
    };
    B64.encode(sign_as(tx, signer).0)
}

#[tokio::test]
async fn health_reports_chain_state() {
    let (status, body) = get(&state(true, None), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chain"], "ready");
    assert_eq!(body["demo_attestor"], false);
    let (status, body) = get(&state(false, None), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chain"], "not_initialised");
}

#[tokio::test]
async fn routes_answer_503_until_the_chain_is_initialised() {
    let s = state(false, None);
    let res = get(&s, &format!("/vaults/{}", b58(AUNT))).await;
    assert_error(
        &res,
        StatusCode::SERVICE_UNAVAILABLE,
        "chain_not_initialised",
    );
}

#[tokio::test]
async fn vault_view_and_lookup_errors() {
    let s = state(true, None);
    let id = seed_vault(ledger(&s), 1, NOW + 120, &[(AUNT, 300)]);
    let (status, body) = get(&s, &format!("/vaults/{}", b58(id))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "open");
    assert_eq!(body["total_contributed"], 300_000_000);
    assert_eq!(body["seconds_left"], 120);
    assert_eq!(body["attestor"], b58(ATTESTOR));
    assert_eq!(body["contributors"][0]["funder"], b58(AUNT));

    let missing = get(&s, &format!("/vaults/{}", b58(UNCLE))).await;
    assert_error(&missing, StatusCode::NOT_FOUND, "vault_not_found");
    let bad = get(&s, "/vaults/not-an-address").await;
    assert_error(&bad, StatusCode::BAD_REQUEST, "invalid_input");
}

#[tokio::test]
async fn student_funder_and_university_views() {
    let s = state(true, None);
    let id = seed_vault(ledger(&s), 1, NOW + 120, &[(AUNT, 300)]);
    let (status, body) = get(&s, &format!("/students/{}/vaults", b58(STUDENT))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["vault"], b58(id));

    let (status, body) = get(&s, &format!("/funders/{}", b58(AUNT))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["contributions"][0]["refund"], "not_due");
    assert_eq!(body["contributions"][0]["vault_status"], "open");

    let (status, body) = get(&s, &format!("/universities/{}/proof", b58(UNIVERSITY))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["funds_held"], 300_000_000);
    assert_eq!(body[0]["attestor"], b58(ATTESTOR));
}

fn new_vault(university: AccountId, deadline: i64) -> Value {
    json!({
        "student": b58(STUDENT), "university": b58(university), "attestor": b58(ATTESTOR),
        "target": 1_000_000_000u64, "deadline": deadline, "nonce": 7,
    })
}

#[tokio::test]
async fn create_vault_then_submit_the_signed_tx() {
    let s = state(true, None);
    let (status, body) = post(&s, "/vaults", new_vault(UNIVERSITY, NOW + 3_600)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["last_valid_block_height"].is_u64());
    let tx = wallet_sign(&body["unsigned_tx"], STUDENT);
    let (status, sent) = post(&s, "/tx/submit", json!({ "tx": tx })).await;
    assert_eq!(status, StatusCode::OK, "{sent}");
    assert!(sent["signature"].is_string());
    let (status, view) = get(&s, &format!("/vaults/{}", body["vault"].as_str().unwrap())).await;
    assert_eq!((status, &view["status"]), (StatusCode::OK, &json!("open")));

    let again = post(&s, "/tx/submit", json!({ "tx": tx })).await;
    assert_error(&again, StatusCode::CONFLICT, "already_applied");
}

#[tokio::test]
async fn create_vault_errors() {
    let s = state(true, None);
    let unregistered = post(&s, "/vaults", new_vault(UNCLE, NOW + 3_600)).await;
    assert_error(
        &unregistered,
        StatusCode::BAD_REQUEST,
        "university_not_registered",
    );
    let mut stranger = new_vault(UNIVERSITY, NOW + 3_600);
    stranger["attestor"] = b58(UNCLE).into();
    let res = post(&s, "/vaults", stranger).await;
    assert_error(&res, StatusCode::BAD_REQUEST, "attestor_not_registered");
    let past = post(&s, "/vaults", new_vault(UNIVERSITY, NOW - 1)).await;
    assert_error(&past, StatusCode::CONFLICT, "DeadlinePassed");
    let malformed = post(&s, "/vaults", json!({ "student": b58(STUDENT) })).await;
    assert_error(&malformed, StatusCode::BAD_REQUEST, "invalid_body");
}

#[tokio::test]
async fn contribute_builds_a_tx_and_maps_rule_errors() {
    let s = state(true, None);
    let open = seed_vault(ledger(&s), 1, NOW + 120, &[]);
    let late = seed_vault(ledger(&s), 2, NOW - 1, &[]);
    let body = |amount: u64| json!({ "funder": b58(AUNT), "amount": amount });

    let (status, built) = post(
        &s,
        &format!("/vaults/{}/contributions", b58(open)),
        body(5_000_000),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{built}");
    let tx = wallet_sign(&built["unsigned_tx"], AUNT);
    assert_eq!(
        post(&s, "/tx/submit", json!({ "tx": tx })).await.0,
        StatusCode::OK
    );
    assert_eq!(
        ledger(&s).stored_vault(open).unwrap().total_contributed(),
        usdc(5)
    );

    let zero = post(&s, &format!("/vaults/{}/contributions", b58(open)), body(0)).await;
    assert_error(&zero, StatusCode::BAD_REQUEST, "ZeroAmount");
    let after = post(
        &s,
        &format!("/vaults/{}/contributions", b58(late)),
        body(5_000_000),
    )
    .await;
    assert_error(&after, StatusCode::CONFLICT, "DeadlinePassed");
}

#[tokio::test]
async fn submit_rejects_bad_transactions() {
    let s = state(true, None);
    let not_b64 = post(&s, "/tx/submit", json!({ "tx": "%%%" })).await;
    assert_error(&not_b64, StatusCode::BAD_REQUEST, "invalid_input");
    let garbage = post(&s, "/tx/submit", json!({ "tx": B64.encode([1, 2, 3]) })).await;
    assert_error(&garbage, StatusCode::BAD_REQUEST, "rejected");
}

#[tokio::test]
async fn demo_attest_is_disabled_by_default() {
    let s = state(true, None);
    let id = seed_vault(ledger(&s), 1, NOW + 120, &[(AUNT, 1_000)]);
    let body = json!({ "vault": b58(id), "outcome": "issued" });
    let res = post(&s, "/demo/attest", body).await;
    assert_error(&res, StatusCode::FORBIDDEN, "demo_attestor_disabled");
    assert_eq!(
        ledger(&s).stored_vault(id).unwrap().status(),
        VaultStatus::Open
    );
}

#[tokio::test]
async fn demo_attest_signs_when_enabled() {
    let s = state(true, Some(ATTESTOR));
    let issued = seed_vault(ledger(&s), 1, NOW + 120, &[(AUNT, 1_000)]);
    let denied = seed_vault(ledger(&s), 2, NOW + 120, &[(AUNT, 10)]);
    let (status, body) = post(
        &s,
        "/demo/attest",
        json!({ "vault": b58(issued), "outcome": "issued" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["demo"], true);
    assert_eq!(
        ledger(&s).stored_vault(issued).unwrap().status(),
        VaultStatus::Closed
    );

    let (status, _) = post(
        &s,
        "/demo/attest",
        json!({ "vault": b58(denied), "outcome": "denied" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        ledger(&s).stored_vault(denied).unwrap().status(),
        VaultStatus::Refunding
    );

    let bad = post(
        &s,
        "/demo/attest",
        json!({ "vault": b58(denied), "outcome": "maybe" }),
    )
    .await;
    assert_error(&bad, StatusCode::BAD_REQUEST, "invalid_body");
}

#[tokio::test]
async fn demo_attest_with_a_key_that_is_not_the_vault_attestor_is_forbidden() {
    let s = state(true, Some(UNCLE));
    let id = seed_vault(ledger(&s), 1, NOW + 120, &[(AUNT, 1_000)]);
    let res = post(
        &s,
        "/demo/attest",
        json!({ "vault": b58(id), "outcome": "denied" }),
    )
    .await;
    assert_error(&res, StatusCode::FORBIDDEN, "Unauthorized");
}

#[tokio::test]
async fn oversized_bodies_are_refused() {
    let s = state(true, None);
    let big = "A".repeat(crate::routes::BODY_LIMIT + 1);
    let res = post(&s, "/tx/submit", json!({ "tx": big })).await;
    assert_error(&res, StatusCode::PAYLOAD_TOO_LARGE, "invalid_body");
}

#[tokio::test]
async fn cors_allows_the_web_origin() {
    let s = state(true, None);
    let req = Request::get("/health")
        .header("origin", WEB)
        .body(Body::empty())
        .unwrap();
    let res = router(s, HeaderValue::from_static(WEB))
        .oneshot(req)
        .await
        .unwrap();
    assert_eq!(res.headers()["access-control-allow-origin"], WEB);
}

#[tokio::test]
async fn cors_refuses_other_origins() {
    let s = state(true, None);
    let req = Request::get("/health")
        .header("origin", "https://evil.example")
        .body(Body::empty())
        .unwrap();
    let res = router(s, HeaderValue::from_static(WEB))
        .oneshot(req)
        .await
        .unwrap();
    // An exact origin is echoed as configured; the browser blocks the mismatch.
    assert_eq!(res.headers()["access-control-allow-origin"], WEB);
}

#[tokio::test]
async fn unknown_routes_get_a_json_404() {
    let res = get(&state(true, None), "/nope").await;
    assert_error(&res, StatusCode::NOT_FOUND, "not_found");
}

#[test]
fn unavailable_hides_upstream_details() {
    crate::error::register_secrets(vec!["api-key=SECRET".into()]);
    let e = ApiError::from(AppError::Unavailable(
        "https://rpc.example/?api-key=SECRET".into(),
    ));
    assert_eq!(e.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(e.message, "upstream unavailable");
    let leaked = ApiError::from(AppError::Rejected("failed at /?api-key=SECRET".into()));
    assert!(!leaked.message.contains("SECRET"));
    assert_eq!(leaked.into_response().status(), StatusCode::BAD_REQUEST);
}
