mod common;

use arrhabon_domain::{AccountId, Contribution, Deadline, DomainError, Vault, VaultStatus};
use common::*;

const ZERO_ID: AccountId = AccountId([0; 32]);

fn create_with(
    id: AccountId,
    student: AccountId,
    university: AccountId,
    attestor: AccountId,
) -> Result<Vault, DomainError> {
    Vault::create(
        id,
        student,
        university,
        attestor,
        usdc(1),
        Deadline::new(DEADLINE),
        NOW,
    )
}

#[test]
fn create_rejects_attestor_equal_to_student() {
    assert_eq!(
        create_with(VAULT, STUDENT, UNIVERSITY, STUDENT),
        Err(DomainError::InvalidParties)
    );
}

#[test]
fn create_rejects_attestor_equal_to_university() {
    assert_eq!(
        create_with(VAULT, STUDENT, UNIVERSITY, UNIVERSITY),
        Err(DomainError::InvalidParties)
    );
}

#[test]
fn create_rejects_university_equal_to_student() {
    assert_eq!(
        create_with(VAULT, STUDENT, STUDENT, ATTESTOR),
        Err(DomainError::InvalidParties)
    );
}

#[test]
fn create_rejects_any_zero_account() {
    for r in [
        create_with(ZERO_ID, STUDENT, UNIVERSITY, ATTESTOR),
        create_with(VAULT, ZERO_ID, UNIVERSITY, ATTESTOR),
        create_with(VAULT, STUDENT, ZERO_ID, ATTESTOR),
        create_with(VAULT, STUDENT, UNIVERSITY, ZERO_ID),
    ] {
        assert_eq!(r, Err(DomainError::InvalidParties));
    }
}

#[test]
fn create_sets_the_vault_id() {
    assert_eq!(open_vault().id(), VAULT);
}

#[test]
fn contribute_to_another_vault_is_rejected() {
    let mut v = open_vault();
    let mut c = Contribution::new(OTHER_VAULT, MOM);
    assert_eq!(
        v.contribute(&mut c, usdc(1), NOW),
        Err(DomainError::WrongVault)
    );
    assert_eq!(v.contributor_count(), 0);
}

#[test]
fn cross_vault_refund_is_rejected() {
    let (mut v, _, _) = funded();
    let mut other = open_vault_with_id(OTHER_VAULT);
    let mut foreign = Contribution::new(OTHER_VAULT, MOM);
    other.contribute(&mut foreign, usdc(600), NOW).unwrap();
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(v.refund(&mut foreign), Err(DomainError::WrongVault));
    assert!(!foreign.refunded());
    assert_eq!(v.refunded_total(), usdc(0));
}

fn open_vault_with_id(id: AccountId) -> Vault {
    Vault::create(
        id,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        usdc(1_000),
        Deadline::new(DEADLINE),
        NOW,
    )
    .unwrap()
}

#[test]
fn close_released_closes_a_released_vault() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    v.close_released().unwrap();
    assert_eq!(v.status(), VaultStatus::Closed);
}

#[test]
fn close_released_requires_released() {
    let (mut v, _, _) = funded();
    assert_eq!(v.close_released(), Err(DomainError::NotReleased));
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(v.close_released(), Err(DomainError::NotReleased));
}

#[test]
fn contribute_one_second_before_deadline_is_accepted() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    v.contribute(&mut c, usdc(1), DEADLINE - 1).unwrap();
    assert_eq!(v.total_contributed(), usdc(1));
}

#[test]
fn attest_issued_one_second_before_deadline_is_accepted() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, DEADLINE - 1).unwrap();
    assert_eq!(v.status(), VaultStatus::Released);
}
