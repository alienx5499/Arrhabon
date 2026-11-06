mod common;

use arrhabon_domain::{
    Contribution, Deadline, DomainError, MAX_VAULT_DURATION_SECS, MIN_CONTRIBUTION, Usdc, Vault,
};
use common::*;

fn create_until(deadline: i64) -> Result<Vault, DomainError> {
    Vault::create(
        VAULT,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        usdc(1_000),
        Deadline::new(deadline),
        NOW,
    )
}

#[test]
fn max_vault_duration_is_365_days() {
    assert_eq!(MAX_VAULT_DURATION_SECS, 365 * 24 * 60 * 60);
}

#[test]
fn create_accepts_deadline_at_the_cap() {
    let v = create_until(NOW + MAX_VAULT_DURATION_SECS).unwrap();
    assert_eq!(v.deadline().unix(), NOW + MAX_VAULT_DURATION_SECS);
}

#[test]
fn create_rejects_deadline_past_the_cap() {
    assert_eq!(
        create_until(NOW + MAX_VAULT_DURATION_SECS + 1),
        Err(DomainError::DeadlineTooFar)
    );
    assert_eq!(create_until(i64::MAX), Err(DomainError::DeadlineTooFar));
}

#[test]
fn min_contribution_is_one_usdc() {
    assert_eq!(MIN_CONTRIBUTION, Usdc::new(1_000_000));
}

#[test]
fn contribute_accepts_exactly_the_minimum() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    v.contribute(&mut c, MIN_CONTRIBUTION, NOW).unwrap();
    assert_eq!(v.total_contributed(), MIN_CONTRIBUTION);
}

#[test]
fn contribute_rejects_below_the_minimum() {
    let mut v = open_vault();
    let mut c = Contribution::new(VAULT, MOM);
    assert_eq!(
        v.contribute(&mut c, Usdc::new(MIN_CONTRIBUTION.micro() - 1), NOW),
        Err(DomainError::BelowMinimum)
    );
    assert_eq!(v.contributor_count(), 0);
    assert_eq!(c.amount(), Usdc::ZERO);
}

#[test]
fn minimum_applies_to_each_top_up() {
    let (mut v, mut mom, _) = funded();
    assert_eq!(
        v.contribute(&mut mom, Usdc::new(1), NOW),
        Err(DomainError::BelowMinimum)
    );
    assert_eq!(mom.amount(), usdc(600));
}
