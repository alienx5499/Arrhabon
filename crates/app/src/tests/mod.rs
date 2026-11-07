//! Use-case tests on the in-memory fakes.

mod commands;
mod crank;
mod faucet;
mod funding;
mod queries;

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use arrhabon_domain::{AccountId, Contribution, Deadline, Usdc, Vault};

use crate::commands::EXPIRY_GRACE_SECS;
use crate::fakes::{FakeLedger, FixedClock};

pub const NOW: i64 = 1_000_000;
pub const STUDENT: AccountId = AccountId([1; 32]);
pub const UNIVERSITY: AccountId = AccountId([2; 32]);
pub const ATTESTOR: AccountId = AccountId([3; 32]);
pub const AUNT: AccountId = AccountId([4; 32]);
pub const UNCLE: AccountId = AccountId([5; 32]);
/// A deadline `ExpireDue` acts on at NOW: past by exactly the grace period.
pub const DUE: i64 = NOW - EXPIRY_GRACE_SECS;

pub fn usdc(whole: u64) -> Usdc {
    Usdc::new(whole * 1_000_000)
}

/// The fakes never wait, so polling once with a no-op waker completes every use case.
pub fn run<F: Future>(fut: F) -> F::Output {
    match pin!(fut).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("fake ports never pend"),
    }
}

pub fn setup() -> (FixedClock, FakeLedger) {
    let clock = FixedClock::at(NOW);
    let ledger = FakeLedger::new(clock.clone());
    ledger.register_university(UNIVERSITY);
    ledger.register_attestor(ATTESTOR);
    (clock, ledger)
}

/// Stores an Open vault (target 1,000 USDC) created just before NOW, with the given contributions.
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
