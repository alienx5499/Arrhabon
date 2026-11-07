//! Route and crank tests on the app fakes.

mod crank;
mod faucet;
mod funding;
mod mirror;
mod registry;
mod routes;

use std::sync::{Arc, OnceLock};

use arrhabon_app::address::to_base58;
use arrhabon_app::fakes::{FakeFaucet, FakeLedger, FakeQuoter, FakeSigner, FixedClock};
use arrhabon_domain::{AccountId, Contribution, Deadline, Usdc, Vault};
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use crate::routes::router;
use crate::state::{AppState, Inner, Ports};

pub struct TestPorts;

impl Ports for TestPorts {
    type Ledger = FakeLedger;
    type Quoter = FakeQuoter;
    type Clock = FixedClock;
    type Attestor = FakeSigner;
    type Faucet = FakeFaucet;
}

pub const NOW: i64 = 1_000_000;
pub const STUDENT: AccountId = AccountId([1; 32]);
pub const UNIVERSITY: AccountId = AccountId([2; 32]);
pub const ATTESTOR: AccountId = AccountId([3; 32]);
pub const AUNT: AccountId = AccountId([4; 32]);
pub const UNCLE: AccountId = AccountId([5; 32]);
pub const WEB: &str = "http://localhost:3000";

pub fn usdc(whole: u64) -> Usdc {
    Usdc::new(whole * 1_000_000)
}

pub fn b58(id: AccountId) -> String {
    to_base58(id)
}

/// A state over fakes; `chain` false leaves the ledger unset, `attestor` enables the DEMO route.
pub fn state(chain: bool, attestor: Option<AccountId>) -> AppState<TestPorts> {
    let clock = FixedClock::at(NOW);
    let ledger = OnceLock::new();
    if chain {
        let fake = FakeLedger::new(clock.clone());
        fake.register_university(UNIVERSITY);
        fake.register_attestor(ATTESTOR);
        let _ = ledger.set(fake);
    }
    AppState(Arc::new(Inner {
        ledger,
        quoter: FakeQuoter::new(clock.clone(), usdc(1)),
        clock,
        attestor: attestor.map(FakeSigner),
        crank: false,
        faucet_on: false,
        faucet: OnceLock::new(),
        faucet_claims: Default::default(),
        aurora_live: false,
        mirror: Default::default(),
        cluster: "devnet",
    }))
}

/// A chain-ready state with the DEMO faucet on, backed by `faucet`.
pub fn faucet_state(faucet: FakeFaucet) -> AppState<TestPorts> {
    let s = state(true, None);
    let inner = Arc::into_inner(s.0).expect("fresh state has one owner");
    let _ = inner.faucet.set(faucet);
    AppState(Arc::new(Inner {
        faucet_on: true,
        ..inner
    }))
}

/// A chain-ready state with the DEMO faucet and mirror config, backed by `faucet`.
pub fn mirror_state(faucet: FakeFaucet, aurora_live: bool) -> AppState<TestPorts> {
    let s = state(true, None);
    let inner = Arc::into_inner(s.0).expect("fresh state has one owner");
    let _ = inner.faucet.set(faucet);
    AppState(Arc::new(Inner {
        faucet_on: true,
        aurora_live,
        ..inner
    }))
}

pub fn ledger(s: &AppState<TestPorts>) -> &FakeLedger {
    s.ledger.get().expect("chain set")
}

/// Stores an Open vault (target 1,000 USDC) with the given contributions (whole USDC).
pub fn seed_vault(
    ledger: &FakeLedger,
    tag: u8,
    deadline: i64,
    funds: &[(AccountId, u64)],
) -> AccountId {
    let id = AccountId([0x50 + tag; 32]);
    let mut vault = Vault::create(
        id,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        usdc(1_000),
        Deadline::new(deadline),
        NOW - 100,
    )
    .expect("valid vault");
    for &(funder, whole) in funds {
        let mut c = Contribution::new(id, funder);
        vault
            .contribute(&mut c, usdc(whole), NOW - 50)
            .expect("valid contribution");
        ledger.put_contribution(c);
    }
    ledger.put_vault(vault);
    id
}

pub async fn send(s: &AppState<TestPorts>, req: Request<Body>) -> (StatusCode, Value) {
    let res = router(s.clone(), HeaderValue::from_static(WEB))
        .oneshot(req)
        .await
        .expect("infallible");
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

pub async fn get(s: &AppState<TestPorts>, uri: &str) -> (StatusCode, Value) {
    send(s, Request::get(uri).body(Body::empty()).unwrap()).await
}

pub async fn post(s: &AppState<TestPorts>, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    send(s, req).await
}
