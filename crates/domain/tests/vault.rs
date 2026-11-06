mod common;

use arrhabon_domain::{Contribution, Deadline, DomainError, Usdc, Vault, VaultStatus};
use common::*;

#[test]
fn create_opens_an_empty_vault() {
    let v = open_vault();
    assert_eq!(v.status(), VaultStatus::Open);
    assert_eq!(v.total_contributed(), Usdc::ZERO);
    assert_eq!(v.contributor_count(), 0);
}

#[test]
fn create_rejects_zero_target() {
    let r = Vault::create(
        VAULT,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        Usdc::ZERO,
        Deadline::new(DEADLINE),
        NOW,
    );
    assert_eq!(r, Err(DomainError::ZeroAmount));
}

#[test]
fn create_rejects_deadline_not_in_future() {
    let r = Vault::create(
        VAULT,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        usdc(1),
        Deadline::new(NOW),
        NOW,
    );
    assert_eq!(r, Err(DomainError::DeadlinePassed));
}

#[test]
fn contribute_records_totals_and_counts_each_funder_once() {
    let (mut v, mut mom, _) = funded();
    v.contribute(&mut mom, usdc(50), NOW).unwrap();
    assert_eq!(mom.amount(), usdc(650));
    assert_eq!(v.total_contributed(), usdc(1_050));
    assert_eq!(v.contributor_count(), 2);
}

#[test]
fn contribute_rejects_zero_amount() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    assert_eq!(
        v.contribute(&mut c, Usdc::ZERO, NOW),
        Err(DomainError::ZeroAmount)
    );
    assert_eq!(v.contributor_count(), 0);
}

#[test]
fn contribute_after_deadline_is_rejected() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    assert_eq!(
        v.contribute(&mut c, usdc(1), DEADLINE),
        Err(DomainError::DeadlinePassed)
    );
}

#[test]
fn contribute_overflow_leaves_state_unchanged() {
    let mut v = open_vault();
    let mut a = Contribution::new(VAULT, MOM);
    let mut b = Contribution::new(VAULT, UNCLE);
    v.contribute(&mut a, Usdc::new(u64::MAX), NOW).unwrap();
    assert_eq!(
        v.contribute(&mut b, usdc(1), NOW),
        Err(DomainError::Overflow)
    );
    assert_eq!(b.amount(), Usdc::ZERO);
    assert_eq!(v.contributor_count(), 1);
}

#[test]
fn contribute_only_while_open() {
    let (mut v, mut mom, _) = funded();
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(
        v.contribute(&mut mom, usdc(1), NOW),
        Err(DomainError::NotOpen)
    );
}

#[test]
fn attest_issued_releases_when_target_met() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    assert_eq!(v.status(), VaultStatus::Released);
}

#[test]
fn release_below_target_is_rejected() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    v.contribute(&mut c, usdc(999), NOW).unwrap();
    assert_eq!(
        v.attest_issued(ATTESTOR, NOW),
        Err(DomainError::BelowTarget)
    );
    assert_eq!(v.status(), VaultStatus::Open);
}

#[test]
fn attest_issued_after_deadline_is_rejected() {
    let (mut v, _, _) = funded();
    assert_eq!(
        v.attest_issued(ATTESTOR, DEADLINE),
        Err(DomainError::DeadlinePassed)
    );
}

#[test]
fn attest_issued_once_refunding_is_rejected() {
    let (mut v, _, _) = funded();
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(v.attest_issued(ATTESTOR, NOW), Err(DomainError::NotOpen));
}

#[test]
fn attestations_require_the_attestor() {
    let (mut v, _, _) = funded();
    assert_eq!(
        v.attest_issued(STUDENT, NOW),
        Err(DomainError::Unauthorized)
    );
    assert_eq!(v.attest_denied(UNIVERSITY), Err(DomainError::Unauthorized));
    assert_eq!(v.status(), VaultStatus::Open);
}

#[test]
fn attest_denied_starts_refunding() {
    let (mut v, _, _) = funded();
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(v.status(), VaultStatus::Refunding);
}

#[test]
fn expire_needs_the_deadline_passed() {
    let (mut v, _, _) = funded();
    assert_eq!(v.expire(DEADLINE - 1), Err(DomainError::DeadlineNotReached));
    v.expire(DEADLINE).unwrap();
    assert_eq!(v.status(), VaultStatus::Refunding);
}

#[test]
fn expire_after_release_is_rejected() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    assert_eq!(v.expire(DEADLINE), Err(DomainError::NotOpen));
}

#[test]
fn empty_vault_closes_directly_instead_of_refunding() {
    let mut v = open_vault();
    v.expire(DEADLINE).unwrap();
    assert_eq!(v.status(), VaultStatus::Closed);
}

#[test]
fn refund_pays_each_funder_once_and_last_refund_closes() {
    let (mut v, mut mom, mut uncle) = funded();
    v.expire(DEADLINE).unwrap();
    v.refund(&mut mom).unwrap();
    assert!(mom.refunded());
    assert_eq!(v.refunded_total(), usdc(600));
    assert_eq!(v.status(), VaultStatus::Refunding);
    v.refund(&mut uncle).unwrap();
    assert_eq!(v.refunded_total(), v.total_contributed());
    assert_eq!(v.status(), VaultStatus::Closed);
}

#[test]
fn double_refund_is_rejected() {
    let (mut v, mut mom, _) = funded();
    v.attest_denied(ATTESTOR).unwrap();
    v.refund(&mut mom).unwrap();
    assert_eq!(v.refund(&mut mom), Err(DomainError::AlreadyRefunded));
    assert_eq!(v.refunded_total(), usdc(600));
}

#[test]
fn refund_before_refunding_is_rejected() {
    let (mut v, mut mom, _) = funded();
    assert_eq!(v.refund(&mut mom), Err(DomainError::NotRefunding));
    v.attest_issued(ATTESTOR, NOW).unwrap();
    assert_eq!(v.refund(&mut mom), Err(DomainError::NotRefunding));
    assert!(!mom.refunded());
}

#[test]
fn refund_of_unrecorded_funder_is_rejected() {
    let (mut v, _, _) = funded();
    v.attest_denied(ATTESTOR).unwrap();
    let mut nobody = Contribution::new(VAULT, STRANGER);
    assert_eq!(v.refund(&mut nobody), Err(DomainError::NothingToRefund));
}

#[test]
fn cancel_by_student_with_only_own_funds() {
    let mut v = open_vault();
    let mut own = Contribution::new(VAULT, STUDENT);
    v.contribute(&mut own, usdc(10), NOW).unwrap();
    v.cancel(STUDENT).unwrap();
    assert_eq!(v.status(), VaultStatus::Refunding);
}

#[test]
fn cancel_after_a_non_student_contribution_is_rejected() {
    let (mut v, _, _) = funded();
    assert_eq!(v.cancel(STUDENT), Err(DomainError::OutsideContributions));
    assert_eq!(v.status(), VaultStatus::Open);
}

#[test]
fn cancel_requires_the_student() {
    let mut v = open_vault();
    assert_eq!(v.cancel(STRANGER), Err(DomainError::Unauthorized));
}

#[test]
fn cancel_only_while_open() {
    let mut v = open_vault();
    v.expire(DEADLINE).unwrap();
    assert_eq!(v.cancel(STUDENT), Err(DomainError::NotOpen));
}
