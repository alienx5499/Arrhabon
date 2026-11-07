//! After settlement: sweeping stray tokens to whoever is owed them, closing accounts for rent.

mod common;

use arrhabon::state::Status;
use common::*;

/// A denied vault with funders `a` (10 USDC) and `b` (20 USDC) not yet refunded.
fn denied() -> (Env, Pubkey, Keypair, Keypair) {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (a, b) = (env.funder(10 * USDC), env.funder(20 * USDC));
    env.contribute(vault, &a, 10 * USDC).unwrap();
    env.contribute(vault, &b, 20 * USDC).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    (env, vault, a, b)
}

/// `denied`, then both funders refunded: the vault is Closed.
fn refunded() -> (Env, Pubkey, Keypair, Keypair) {
    let (mut env, vault, a, b) = denied();
    env.refund(vault, key(&a), &a).unwrap();
    env.refund(vault, key(&b), &b).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Closed);
    (env, vault, a, b)
}

/// Tokens sent straight to the vault's token account, outside `contribute`.
fn send_stray(env: &mut Env, vault: Pubkey, amount: u64) {
    let vault_token = ata(&vault, &env.mint);
    let held = env.balance(&vault_token);
    env.token_account(vault_token, &vault, held + amount);
}

#[test]
fn sweep_after_refunds_pays_stray_tokens_to_the_student() {
    let (mut env, vault, a, _) = refunded();
    send_stray(&mut env, vault, 3 * USDC);
    let student_ata = ata(&key(&env.student), &env.mint);
    assert!(!env.exists(&student_ata));
    env.sweep(vault, &a, key(&env.student)).unwrap();
    assert_eq!(env.balance(&student_ata), 3 * USDC);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 0);
}

#[test]
fn sweep_before_every_funder_is_refunded_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(10 * USDC);
    env.contribute(vault, &a, 10 * USDC).unwrap();
    assert_rejected(env.sweep(vault, &a, key(&env.student)), "NotClosed");
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    assert_rejected(env.sweep(vault, &a, key(&env.student)), "NotClosed");
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 10 * USDC);
}

#[test]
fn stray_tokens_after_release_go_to_the_university_then_the_vault_closes() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    send_stray(&mut env, vault, USDC);
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "BalanceRemaining");
    let u = key(&env.university);
    env.sweep(vault, &a, u).unwrap();
    assert_eq!(env.balance(&ata(&u, &env.mint)), TARGET + USDC);
    env.close_contribution(vault, key(&a), &a).unwrap();
    env.close_vault(vault, &student).unwrap();
    assert!(!env.exists(&vault));
}

#[test]
fn sweep_to_anyone_but_the_entitled_party_rejected() {
    let (mut env, vault, a, _) = refunded();
    send_stray(&mut env, vault, USDC);
    // Refund path: the student is owed the strays, not the university or a funder.
    for wrong in [key(&env.university), key(&a)] {
        let accounts = env.sweep_accounts(vault, &a, wrong);
        assert_rejected(env.send_sweep(accounts, &a), "Unauthorized");
    }
    assert_eq!(env.balance(&ata(&vault, &env.mint)), USDC);
}

#[test]
fn sweep_after_release_to_the_student_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    send_stray(&mut env, vault, USDC);
    assert_rejected(env.sweep(vault, &a, key(&env.student)), "Unauthorized");
    assert_eq!(env.balance(&ata(&vault, &env.mint)), USDC);
}

#[test]
fn closes_return_rent_to_each_payer() {
    let (mut env, vault, a, b) = refunded();
    let s = key(&env.student);
    let (vault_token, contribution) = (ata(&vault, &env.mint), contribution_pda(&vault, &key(&a)));
    let (contribution_rent, vault_rent) = (env.lamports(&contribution), env.lamports(&vault));
    let token_rent = env.lamports(&vault_token);
    let (a_before, student_before) = (env.lamports(&key(&a)), env.lamports(&s));

    // b pays the fee for closing a's record; the rent still goes to a.
    env.close_contribution(vault, key(&a), &b).unwrap();
    env.close_contribution(vault, key(&b), &b).unwrap();
    assert_eq!(env.lamports(&key(&a)), a_before + contribution_rent);
    assert!(!env.exists(&contribution));

    let student = env.student.insecure_clone();
    env.close_vault(vault, &student).unwrap();
    assert!(!env.exists(&vault) && !env.exists(&vault_token));
    // The student signs, so pays one 5,000-lamport fee.
    assert_eq!(
        env.lamports(&s),
        student_before + vault_rent + token_rent - 5_000
    );
}

#[test]
fn close_vault_while_open_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "NotClosed");
}

#[test]
fn close_vault_while_a_funder_is_unrefunded_rejected() {
    let (mut env, vault, a, _) = denied();
    env.refund(vault, key(&a), &a).unwrap();
    env.close_contribution(vault, key(&a), &a).unwrap();
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "NotClosed");
    assert_eq!(env.vault(&vault).status, Status::Refunding);
}

fn student_of(env: &Env) -> Keypair {
    env.student.insecure_clone()
}

#[test]
fn close_vault_with_a_balance_rejected_until_swept() {
    let (mut env, vault, a, b) = refunded();
    env.close_contribution(vault, key(&a), &a).unwrap();
    env.close_contribution(vault, key(&b), &b).unwrap();
    send_stray(&mut env, vault, 1);
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "BalanceRemaining");
    env.sweep(vault, &a, key(&env.student)).unwrap();
    env.close_vault(vault, &student).unwrap();
}

#[test]
fn close_vault_with_open_contributions_rejected() {
    let (mut env, vault, a, _) = refunded();
    env.close_contribution(vault, key(&a), &a).unwrap();
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "ContributionsOpen");
    assert!(env.exists(&vault));
}

#[test]
fn close_vault_by_non_student_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    env.cancel(vault, &student_of(&env)).unwrap();
    let thief = env.funder(0);
    assert_rejected(env.close_vault(vault, &thief), "ConstraintHasOne");
    assert!(env.exists(&vault));
}

#[test]
fn close_unrefunded_contribution_rejected() {
    let (mut env, vault, a, _) = denied();
    assert_rejected(env.close_contribution(vault, key(&a), &a), "NotClosed");
    assert!(env.exists(&contribution_pda(&vault, &key(&a))));
}

#[test]
fn close_contribution_while_open_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(USDC);
    env.contribute(vault, &a, USDC).unwrap();
    assert_rejected(env.close_contribution(vault, key(&a), &a), "NotClosed");
}

#[test]
fn close_someone_elses_contribution_to_yourself_rejected() {
    let (mut env, vault, a, b) = refunded();
    let a_record = contribution_pda(&vault, &key(&a));
    assert_rejected(
        env.close_contribution_with(vault, key(&b), a_record, &b),
        "ConstraintSeeds",
    );
    assert!(env.exists(&a_record));
}

#[test]
fn released_vault_closes_after_its_contributions() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    let student = student_of(&env);
    assert_rejected(env.close_vault(vault, &student), "ContributionsOpen");
    env.close_contribution(vault, key(&a), &a).unwrap();
    env.close_vault(vault, &student).unwrap();
    assert!(!env.exists(&vault));
}

/// A closed vault's address can be reused for a new vault, which starts with no records.
#[test]
fn vault_recreated_at_a_closed_address_starts_clean() {
    let (mut env, vault, a, b) = refunded();
    env.close_contribution(vault, key(&a), &a).unwrap();
    env.close_contribution(vault, key(&b), &b).unwrap();
    let student = student_of(&env);
    env.close_vault(vault, &student).unwrap();
    assert_eq!(env.create_vault(0), vault);
    let v = env.vault(&vault);
    assert_eq!((v.status, v.contributor_count), (Status::Open, 0));
    assert!(!env.exists(&contribution_pda(&vault, &key(&a))));
}
