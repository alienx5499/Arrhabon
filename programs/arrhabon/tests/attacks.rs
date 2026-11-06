//! Account-substitution attacks and the minimum, deadline-cap and settlement-order rules.

mod common;

use anchor_spl::token::ID as TOKEN_PROGRAM;
use arrhabon::state::Status;
use arrhabon_domain::MAX_VAULT_DURATION_SECS;
use common::*;

/// A vault funded to `TARGET` by one funder.
fn funded() -> (Env, Pubkey) {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    (env, vault)
}

#[test]
fn contribute_into_non_ata_vault_token_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(10 * USDC);
    // Right mint, owned by the vault PDA, but not its ATA.
    let fake = Pubkey::new_unique();
    env.token_account(fake, &vault, 0);
    let mut accounts = env.contribute_accounts(vault, &a);
    accounts.vault_token = fake;
    assert_rejected(
        env.send_contribute(accounts, &a, 10 * USDC),
        "ConstraintAssociated",
    );
    assert_eq!(env.balance(&fake), 0);
}

#[test]
fn contribute_with_other_mint_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(0);
    let other = Pubkey::new_unique();
    env.put_mint(other, 6, TOKEN_PROGRAM);
    let (from, to) = (ata(&key(&a), &other), ata(&vault, &other));
    env.token_account_of(from, &other, &key(&a), 10 * USDC);
    env.token_account_of(to, &other, &vault, 0);
    let mut accounts = env.contribute_accounts(vault, &a);
    (accounts.mint, accounts.funder_token, accounts.vault_token) = (other, from, to);
    assert_rejected(
        env.send_contribute(accounts, &a, 10 * USDC),
        "ConstraintHasOne",
    );
    assert_eq!(env.vault(&vault).total_contributed, 0);
}

#[test]
fn confirm_from_vault_token_of_other_mint_rejected() {
    let (mut env, vault) = funded();
    let other = Pubkey::new_unique();
    env.put_mint(other, 6, TOKEN_PROGRAM);
    let wrong = ata(&vault, &other);
    env.token_account_of(wrong, &other, &vault, TARGET);
    let attestor = env.attestor.insecure_clone();
    let mut accounts = env.confirm_accounts(vault, &attestor, key(&env.university));
    accounts.vault_token = wrong;
    assert_rejected(
        env.send_confirm(accounts, &attestor),
        "ConstraintAssociated",
    );
    assert_eq!(env.vault(&vault).status, Status::Open);
}

#[test]
fn confirm_to_non_ata_university_account_rejected() {
    let (mut env, vault) = funded();
    let u = key(&env.university);
    let side = Pubkey::new_unique();
    env.token_account(side, &u, 0);
    let attestor = env.attestor.insecure_clone();
    let mut accounts = env.confirm_accounts(vault, &attestor, u);
    accounts.university_token = side;
    assert_rejected(
        env.send_confirm(accounts, &attestor),
        "AccountNotAssociatedTokenAccount",
    );
    assert_eq!(env.balance(&ata(&vault, &env.mint)), TARGET);
}

#[test]
fn confirm_to_university_ata_of_attacker_rejected() {
    let (mut env, vault) = funded();
    let thief = env.funder(0);
    let attestor = env.attestor.insecure_clone();
    let mut accounts = env.confirm_accounts(vault, &attestor, key(&env.university));
    accounts.university_token = ata(&key(&thief), &env.mint);
    assert_rejected(
        env.send_confirm(accounts, &attestor),
        "ConstraintTokenOwner",
    );
    assert_eq!(env.balance(&ata(&key(&thief), &env.mint)), 0);
}

#[test]
fn contribution_below_minimum_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(USDC);
    assert_rejected(env.contribute(vault, &a, USDC - 1), "BelowMinimum");
    env.contribute(vault, &a, USDC).unwrap();
    assert_eq!(env.vault(&vault).total_contributed, USDC);
}

#[test]
fn deadline_above_cap_rejected() {
    let mut env = Env::new();
    let cap = START + MAX_VAULT_DURATION_SECS;
    assert_rejected(env.try_create_vault(0, TARGET, cap + 1), "DeadlineTooFar");
    env.try_create_vault(1, TARGET, cap).unwrap();
}

/// Current rule: the attestor may deny after the deadline. Either way the outcome is the
/// same Refunding state `expire` would produce, so no money can move anywhere new.
#[test]
fn deny_after_deadline_starts_refunds() {
    let (mut env, vault) = funded();
    env.warp(DEADLINE + 1);
    let attestor = env.attestor.insecure_clone();
    env.deny(vault, &attestor).unwrap();
    assert_eq!(env.vault(&vault).status, Status::Refunding);
    let crank = env.funder(0);
    assert_rejected(env.expire(vault, &crank), "NotOpen");
}

#[test]
fn expire_after_release_rejected() {
    let (mut env, vault) = funded();
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    env.warp(DEADLINE);
    let crank = env.funder(0);
    assert_rejected(env.expire(vault, &crank), "NotOpen");
    assert_eq!(env.vault(&vault).status, Status::Closed);
}

#[test]
fn pre_created_vault_ata_does_not_block_create() {
    let mut env = Env::new();
    let vault = vault_pda(&key(&env.student), 0);
    // Griefer sends tokens to the predictable ATA before the student creates the vault.
    env.token_account(ata(&vault, &env.mint), &vault, 3 * USDC);
    assert_eq!(env.create_vault(0), vault);
    assert_eq!(env.vault(&vault).status, Status::Open);
}

#[test]
fn confirm_creates_missing_university_ata_paid_by_attestor() {
    let (mut env, vault) = funded();
    let university_ata = ata(&key(&env.university), &env.mint);
    assert!(!env.exists(&university_ata));
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    assert_eq!(env.balance(&university_ata), TARGET);
    assert_eq!(env.vault(&vault).status, Status::Closed);
}

#[test]
fn confirm_pays_whole_token_balance_including_direct_transfers() {
    let mut env = Env::new();
    let vault = vault_pda(&key(&env.student), 0);
    env.token_account(ata(&vault, &env.mint), &vault, 3 * USDC);
    env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    let attestor = env.attestor.insecure_clone();
    env.confirm(vault, &attestor).unwrap();
    let university_ata = ata(&key(&env.university), &env.mint);
    assert_eq!(env.balance(&university_ata), TARGET + 3 * USDC);
    assert_eq!(env.balance(&ata(&vault, &env.mint)), 0);
    assert_eq!(env.vault(&vault).total_contributed, TARGET);
}
