mod common;

use arrhabon_domain::{Usdc, Vault};
use common::*;

fn held(v: &Vault) -> Usdc {
    v.funds_held()
}

#[test]
fn open_vault_holds_everything_contributed() {
    let (v, _, _) = funded();
    assert_eq!(held(&v), usdc(1_000));
    assert_eq!(held(&open_vault()), Usdc::ZERO);
}

#[test]
fn partly_refunded_vault_holds_only_what_is_still_owed() {
    let (mut v, mut mom, mut uncle) = funded();
    v.expire(DEADLINE).unwrap();
    v.refund(&mut mom).unwrap();
    assert_eq!(held(&v), usdc(400));
    v.refund(&mut uncle).unwrap();
    assert_eq!(held(&v), Usdc::ZERO);
}

#[test]
fn released_and_closed_vaults_hold_nothing() {
    let (mut v, _, _) = funded();
    v.attest_issued(ATTESTOR, NOW).unwrap();
    assert_eq!(held(&v), Usdc::ZERO);
    v.close_released().unwrap();
    assert_eq!(held(&v), Usdc::ZERO);
}

#[test]
fn accepts_contributions_only_while_open_and_before_the_deadline() {
    use arrhabon_domain::DomainError;
    let (mut v, _, _) = funded();
    assert_eq!(v.accepts_contributions(DEADLINE - 1), Ok(()));
    assert_eq!(
        v.accepts_contributions(DEADLINE),
        Err(DomainError::DeadlinePassed)
    );
    v.attest_denied(ATTESTOR).unwrap();
    assert_eq!(v.accepts_contributions(NOW), Err(DomainError::NotOpen));
}
