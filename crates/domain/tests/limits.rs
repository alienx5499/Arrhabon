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
    let mut v = open_vault();
    let mut mom = Contribution::new(VAULT, MOM);
    v.contribute(&mut mom, usdc(600), NOW).unwrap();
    assert_eq!(
        v.contribute(&mut mom, Usdc::new(1), NOW),
        Err(DomainError::BelowMinimum)
    );
    assert_eq!(mom.amount(), usdc(600));
}

/// MOM has paid 600 of the 1,000 target, so 400 remain.
fn part_funded() -> (Vault, Contribution) {
    let mut v = open_vault();
    let mut mom = Contribution::new(VAULT, MOM);
    v.contribute(&mut mom, usdc(600), NOW).unwrap();
    (v, Contribution::new(VAULT, UNCLE))
}

#[test]
fn contribute_accepts_exactly_the_remainder() {
    let (mut v, mut uncle) = part_funded();
    v.contribute(&mut uncle, usdc(400), NOW).unwrap();
    assert_eq!(v.total_contributed(), v.target());
}

#[test]
fn contribute_above_the_remainder_is_rejected_and_changes_nothing() {
    let (mut v, mut uncle) = part_funded();
    let over = Usdc::new(usdc(400).micro() + 1);
    assert_eq!(
        v.contribute(&mut uncle, over, NOW),
        Err(DomainError::ExceedsTarget)
    );
    assert_eq!(uncle.amount(), Usdc::ZERO);
    assert_eq!(v.total_contributed(), usdc(600));
    assert_eq!(v.contributor_count(), 1);
}

#[test]
fn contribute_to_a_fully_funded_vault_is_rejected() {
    let (mut v, mut mom, _) = funded();
    assert_eq!(
        v.contribute(&mut mom, usdc(1), NOW),
        Err(DomainError::ExceedsTarget)
    );
    assert_eq!(v.total_contributed(), v.target());
}

#[test]
fn a_remainder_below_the_minimum_may_be_paid_exactly() {
    let half = Usdc::new(MIN_CONTRIBUTION.micro() / 2);
    let mut v = open_vault();
    let mut mom = Contribution::new(VAULT, MOM);
    let mut uncle = Contribution::new(VAULT, UNCLE);
    v.contribute(&mut mom, usdc(1_000).checked_sub(half).unwrap(), NOW)
        .unwrap();
    let less = Usdc::new(half.micro() - 1);
    let more = Usdc::new(half.micro() + 1);
    assert_eq!(
        v.contribute(&mut uncle, less, NOW),
        Err(DomainError::BelowMinimum)
    );
    assert_eq!(
        v.contribute(&mut uncle, more, NOW),
        Err(DomainError::ExceedsTarget)
    );
    v.contribute(&mut uncle, half, NOW).unwrap();
    assert_eq!(v.total_contributed(), v.target());
}
