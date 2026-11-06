//! Money paths: release to the university, refunds after deny, expiry and cancel.

mod common;

use arrhabon::state::Status;
use common::*;

#[test]
fn release_pays_university_the_whole_total_and_closes() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (a, b) = (env.funder(600_000_000), env.funder(500_000_000));
    env.contribute(vault, &a, 600_000_000).unwrap();
    env.contribute(vault, &b, 500_000_000).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    env.token_account(university_ata, &key(&env.university), 7);

    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();

    assert_eq!(env.balance(&university_ata), 7 + 1_100_000_000);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 0);
    let v = env.vault(&vault);
    assert_eq!(v.status, Status::Closed);
    assert_eq!(v.total_contributed, 1_100_000_000);
    assert_eq!(v.contributor_count, 2);
}

#[test]
fn deny_refunds_each_funder_exactly_then_closes() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (a, b) = (env.funder(300_000_000), env.funder(200_000_000));
    env.contribute(vault, &a, 250_000_000).unwrap();
    env.contribute(vault, &b, 200_000_000).unwrap();
    env.contribute(vault, &a, 50_000_000).unwrap();
    assert_eq!(env.contribution(&vault, &key(&a)).amount, 300_000_000);

    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Refunding);

    let crank = env.funder(0);
    env.refund(vault, key(&a), &crank).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Refunding);
    env.refund(vault, key(&b), &crank).unwrap();

    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 300_000_000);
    assert_eq!(env.balance(&ata(&key(&b), &env.mint)), 200_000_000);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 0);
    let v = env.vault(&vault);
    assert_eq!(v.status, Status::Closed);
    assert_eq!(v.refunded_total, v.total_contributed);
    assert!(env.contribution(&vault, &key(&a)).refunded);
}

#[test]
fn expiry_refunds_each_funder_exactly_then_closes() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (a, b) = (env.funder(400_000_000), env.funder(100_000_000));
    env.contribute(vault, &a, 400_000_000).unwrap();
    env.contribute(vault, &b, 100_000_000).unwrap();
    let crank = env.funder(0);

    assert_rejected(env.expire(vault, &crank), "DeadlineNotReached");
    env.warp(DEADLINE);
    env.expire(vault, &crank).unwrap();
    env.refund(vault, key(&b), &crank).unwrap();
    env.refund(vault, key(&a), &crank).unwrap();

    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 400_000_000);
    assert_eq!(env.balance(&ata(&key(&b), &env.mint)), 100_000_000);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 0);
    assert_eq!(env.vault(&vault).status, Status::Closed);
}

#[test]
fn refund_creates_missing_funder_ata_paid_by_caller() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let funder = Keypair::new();
    let f = key(&funder);
    env.airdrop(&f);
    // Funds come from a non-ATA token account, so the funder's ATA never exists.
    let source = Pubkey::new_unique();
    env.token_account(source, &f, 90_000_000);
    env.contribute_with(
        vault,
        &funder,
        90_000_000,
        source,
        contribution_pda(&vault, &f),
    )
    .unwrap();
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    let funder_ata = ata(&f, &env.mint);
    assert!(!env.exists(&funder_ata));

    let crank = env.funder(0);
    env.refund(vault, f, &crank).unwrap();

    assert_eq!(env.balance(&funder_ata), 90_000_000);
    assert_eq!(env.vault(&vault).status, Status::Closed);
}

#[test]
fn double_refund_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (a, b) = (env.funder(10 * USDC), env.funder(20 * USDC));
    env.contribute(vault, &a, 10 * USDC).unwrap();
    env.contribute(vault, &b, 20 * USDC).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    let crank = env.funder(0);
    env.refund(vault, key(&a), &crank).unwrap();

    assert_rejected(env.refund(vault, key(&a), &crank), "AlreadyRefunded");
    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 10 * USDC);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 20 * USDC);
}

#[test]
fn refund_to_non_funder_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(10 * USDC);
    env.contribute(vault, &a, 10 * USDC).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    let stranger = env.funder(0);

    assert_rejected(
        env.refund(vault, key(&stranger), &stranger),
        "AccountNotInitialized",
    );
    // Recorded funder's contribution, but paid to someone else: seeds bind payee to funder.
    let a_contribution = contribution_pda(&vault, &key(&a));
    assert_rejected(
        env.refund_with(vault, key(&stranger), a_contribution, &stranger),
        "ConstraintSeeds",
    );
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 10 * USDC);
}

#[test]
fn refund_while_open_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(10 * USDC);
    env.contribute(vault, &a, 10 * USDC).unwrap();
    assert_rejected(env.refund(vault, key(&a), &a), "NotRefunding");
}

#[test]
fn cancel_with_only_student_money_refunds_student() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let s = key(&env.student);
    let student_ata = ata(&s, &env.mint);
    env.token_account(student_ata, &s, 50 * USDC);
    let student = env.student.insecure_clone();
    env.contribute(vault, &student, 50 * USDC).unwrap();

    env.cancel(vault, &student).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Refunding);
    env.refund(vault, s, &student).unwrap();

    assert_eq!(env.balance(&student_ata), 50 * USDC);
    assert_eq!(env.vault(&vault).status, Status::Closed);
}

#[test]
fn cancel_empty_vault_closes() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let student = env.student.insecure_clone();
    env.cancel(vault, &student).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Closed);
}
