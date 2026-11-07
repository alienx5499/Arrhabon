mod common;

use arrhabon_domain::{Contribution, DomainError, Usdc, VaultStatus};
use common::*;

#[test]
fn sweep_after_refunds_pays_the_student() {
    let (mut v, mut mom, mut uncle) = funded();
    assert_eq!(v.sweep(), Err(DomainError::NotClosed));
    v.attest_denied(ATTESTOR).unwrap();
    v.refund(&mut mom).unwrap();
    assert_eq!(v.sweep(), Err(DomainError::NotClosed));
    v.refund(&mut uncle).unwrap();
    assert_eq!(v.sweep(), Ok(STUDENT));
}

#[test]
fn sweep_of_an_empty_closed_vault_is_allowed() {
    let mut v = open_vault();
    v.cancel(STUDENT).unwrap();
    assert_eq!(v.status(), VaultStatus::Closed);
    assert_eq!(v.sweep(), Ok(STUDENT));
}

#[test]
fn sweep_after_release_pays_the_university() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    v.close_released().unwrap();
    assert_eq!(v.sweep(), Ok(UNIVERSITY));
}

#[test]
fn refunded_contribution_may_close_while_others_are_still_owed() {
    let (mut v, mut mom, uncle) = funded();
    v.expire(DEADLINE).unwrap();
    assert_eq!(v.close_contribution(&mom), Err(DomainError::NotClosed));
    v.refund(&mut mom).unwrap();
    v.close_contribution(&mom).unwrap();
    assert_eq!(v.closed_contributions(), 1);
    assert_eq!(v.close_contribution(&uncle), Err(DomainError::NotClosed));
}

#[test]
fn contribution_cannot_close_while_the_vault_is_open() {
    let (mut v, mom, _) = funded();
    assert_eq!(v.close_contribution(&mom), Err(DomainError::NotClosed));
    assert_eq!(v.closed_contributions(), 0);
}

#[test]
fn contributions_of_a_released_vault_may_close() {
    let (mut v, mom, uncle) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    v.close_released().unwrap();
    v.close_contribution(&mom).unwrap();
    v.close_contribution(&uncle).unwrap();
    assert_eq!(v.closed_contributions(), 2);
}

#[test]
fn contribution_of_another_vault_cannot_close() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    v.close_released().unwrap();
    let foreign = Contribution::new(OTHER_VAULT, MOM);
    assert_eq!(v.close_contribution(&foreign), Err(DomainError::WrongVault));
}

#[test]
fn vault_closes_only_when_closed_empty_and_every_contribution_closed() {
    let (mut v, mut mom, mut uncle) = funded();
    assert_eq!(v.close(Usdc::ZERO), Err(DomainError::NotClosed));
    v.attest_denied(ATTESTOR).unwrap();
    v.refund(&mut mom).unwrap();
    assert_eq!(v.close(Usdc::ZERO), Err(DomainError::NotClosed));
    v.refund(&mut uncle).unwrap();
    assert_eq!(v.close(usdc(1)), Err(DomainError::BalanceRemaining));
    v.close_contribution(&mom).unwrap();
    assert_eq!(v.close(Usdc::ZERO), Err(DomainError::ContributionsOpen));
    v.close_contribution(&uncle).unwrap();
    assert_eq!(v.close(Usdc::ZERO), Ok(()));
}

#[test]
fn empty_closed_vault_closes_at_once() {
    let mut v = open_vault();
    v.expire(DEADLINE).unwrap();
    assert_eq!(v.close(Usdc::ZERO), Ok(()));
}

#[test]
fn sweep_is_rejected_while_open_or_refunding() {
    let (mut v, mut mom, _) = funded();
    assert_eq!(v.sweep(), Err(DomainError::NotClosed));
    v.expire(DEADLINE).unwrap();
    v.refund(&mut mom).unwrap();
    assert_eq!(v.sweep(), Err(DomainError::NotClosed));
}
