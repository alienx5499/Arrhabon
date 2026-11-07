//! Rejections: wrong signers, deadline rules, cross-vault accounts, cancel rules.

mod common;

use arrhabon::state::Status;
use common::*;

#[test]
fn wrong_attestor_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    env.token_account(university_ata, &key(&env.university), 0);
    let student = env.student.insecure_clone();

    assert_rejected(env.confirm(vault, &a), "Unauthorized");
    assert_rejected(env.confirm(vault, &student), "Unauthorized");
    assert_rejected(env.deny(vault, &student), "Unauthorized");
    assert_eq!(env.balance(&university_ata), 0);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), TARGET);
    assert_eq!(env.vault(&vault).status, Status::Open);
}

#[test]
fn confirm_to_other_university_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let thief = env.funder(0);
    let attestor = env.attestor.insecure_clone();

    assert_rejected(
        env.confirm_to(vault, &attestor, key(&thief)),
        "ConstraintHasOne",
    );
    assert_eq!(env.balance(&ata(&vault, &env.mint)), TARGET);
}

#[test]
fn confirm_below_target_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET - 1);
    env.contribute(vault, &a, TARGET - 1).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    env.token_account(university_ata, &key(&env.university), 0);
    let attestor = env.attestor.insecure_clone();
    assert_rejected(env.confirm(vault, &attestor), "BelowTarget");
}

#[test]
fn contribute_after_deadline_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(100 * USDC);
    env.warp(DEADLINE);
    assert_rejected(env.contribute(vault, &a, 100 * USDC), "DeadlinePassed");
    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 100 * USDC);
}

#[test]
fn confirm_after_deadline_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    env.token_account(university_ata, &key(&env.university), 0);
    env.warp(DEADLINE);
    let attestor = env.attestor.insecure_clone();

    assert_rejected(env.confirm(vault, &attestor), "DeadlinePassed");
    assert_eq!(env.balance(&university_ata), 0);
}

#[test]
fn confirm_once_refunding_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    env.token_account(university_ata, &key(&env.university), 0);
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    assert_rejected(env.confirm(vault, &attestor), "NotOpen");
}

#[test]
fn cross_vault_contribution_account_rejected() {
    let mut env = Env::new();
    let (va, vb) = (env.create_vault(0), env.create_vault(1));
    let a = env.funder(30 * USDC);
    env.contribute(va, &a, 10 * USDC).unwrap();
    env.contribute(vb, &a, 20 * USDC).unwrap();
    let f = key(&a);
    let from = ata(&f, &env.mint);

    assert_rejected(
        env.contribute_with(va, &a, 5 * USDC, from, contribution_pda(&vb, &f)),
        "ConstraintSeeds",
    );
    let attestor = env.attestor.insecure_clone();
    env.deny(va, &attestor).unwrap();
    assert_rejected(
        env.refund_with(va, f, contribution_pda(&vb, &f), &a),
        "ConstraintSeeds",
    );
    assert_eq!(env.balance(&ata(&va, &env.mint)), 10 * USDC);
    assert_eq!(env.balance(&ata(&vb, &env.mint)), 20 * USDC);
}

#[test]
fn cancel_by_non_student_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let attestor = env.attestor.insecure_clone();
    assert_rejected(env.cancel(vault, &attestor), "Unauthorized");
    assert_eq!(env.vault(&vault).status, Status::Open);
}

#[test]
fn create_rejects_invalid_terms() {
    let mut env = Env::new();
    assert_rejected(env.try_create_vault(0, 0, DEADLINE), "ZeroAmount");
    assert_rejected(env.try_create_vault(1, TARGET, START), "DeadlinePassed");
}

#[test]
fn zero_contribution_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(10 * USDC);
    assert_rejected(env.contribute(vault, &a, 0), "ZeroAmount");
}
