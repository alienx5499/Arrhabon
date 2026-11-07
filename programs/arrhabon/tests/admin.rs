//! Config and registry: only the upgrade authority configures, only the admin onboards,
//! and a student can only name registered, active parties.

mod common;

use anchor_spl::token::ID as TOKEN_PROGRAM;
use arrhabon::state::{Config, University};
use common::*;

fn bare_with_config() -> Env {
    let mut env = Env::bare();
    let (admin, mint) = (env.admin.insecure_clone(), env.mint);
    env.init_config(&admin, mint).unwrap();
    env
}

#[test]
fn init_config_stores_admin_and_mint() {
    let env = bare_with_config();
    let config: Config = env.read(&config_pda());
    assert_eq!(config.admin, key(&env.admin));
    assert_eq!(config.mint, env.mint);
}

#[test]
fn init_config_by_non_upgrade_authority_rejected() {
    let mut env = Env::bare();
    let mint = env.mint;
    let impostor = env.funder(0);
    assert_rejected(env.init_config(&impostor, mint), "Unauthorized");
    assert!(!env.exists(&config_pda()));
}

#[test]
fn init_config_twice_rejected() {
    let mut env = bare_with_config();
    let (admin, mint) = (env.admin.insecure_clone(), env.mint);
    let other = Pubkey::new_unique();
    env.put_mint(other, 6, TOKEN_PROGRAM);
    let logs = env.init_config(&admin, other).unwrap_err();
    assert!(logs.contains("already in use"), "{logs}");
    assert_eq!(env.read::<Config>(&config_pda()).mint, mint);
}

#[test]
fn init_config_rejects_token_2022_mint() {
    let mut env = Env::bare();
    let admin = env.admin.insecure_clone();
    let mint_2022 = Pubkey::new_unique();
    env.put_mint(mint_2022, 6, anchor_spl::token_2022::ID);
    assert_rejected(
        env.init_config(&admin, mint_2022),
        "AccountOwnedByWrongProgram",
    );
}

#[test]
fn init_config_rejects_wrong_decimals() {
    let mut env = Env::bare();
    let admin = env.admin.insecure_clone();
    let mint9 = Pubkey::new_unique();
    env.put_mint(mint9, 9, TOKEN_PROGRAM);
    assert_rejected(env.init_config(&admin, mint9), "WrongDecimals");
}

#[test]
fn register_by_non_admin_rejected() {
    let mut env = bare_with_config();
    let impostor = env.funder(0);
    let wallet = Pubkey::new_unique();
    assert_rejected(env.register_university(&impostor, wallet), "Unauthorized");
    assert_rejected(env.register_attestor(&impostor, wallet), "Unauthorized");
    assert!(!env.exists(&university_pda(&wallet)));
    assert!(!env.exists(&attestor_pda(&wallet)));
}

#[test]
fn set_active_by_non_admin_rejected() {
    let mut env = Env::new();
    let impostor = env.funder(0);
    let (u, a) = (key(&env.university), key(&env.attestor));
    assert_rejected(
        env.set_university_active(&impostor, u, false),
        "Unauthorized",
    );
    assert_rejected(env.set_attestor_active(&impostor, a, false), "Unauthorized");
    assert!(env.read::<University>(&university_pda(&u)).active);
}

#[test]
fn create_with_unregistered_university_rejected() {
    let mut env = Env::new();
    let terms = Terms {
        university: Pubkey::new_unique(),
        ..env.terms()
    };
    let accounts = env.create_accounts(0, &terms);
    assert_rejected(env.send_create(0, terms, accounts), "AccountNotInitialized");
}

#[test]
fn create_with_unregistered_attestor_rejected() {
    let mut env = Env::new();
    // The student appointing themselves (or a friend) as attestor is not registered.
    let terms = Terms {
        attestor: Pubkey::new_unique(),
        ..env.terms()
    };
    let accounts = env.create_accounts(0, &terms);
    assert_rejected(env.send_create(0, terms, accounts), "AccountNotInitialized");
}

#[test]
fn create_with_registry_entry_of_another_wallet_rejected() {
    let mut env = Env::new();
    let fake = Pubkey::new_unique();
    let terms = Terms {
        attestor: fake,
        ..env.terms()
    };
    // Real attestor's entry offered for an unregistered attestor wallet: seeds mismatch.
    let mut accounts = env.create_accounts(0, &terms);
    accounts.registered_attestor = attestor_pda(&key(&env.attestor));
    assert_rejected(env.send_create(0, terms, accounts), "ConstraintSeeds");
}

#[test]
fn create_with_inactive_university_rejected() {
    let mut env = Env::new();
    let (admin, u) = (env.admin.insecure_clone(), key(&env.university));
    env.set_university_active(&admin, u, false).unwrap();
    assert_rejected(
        env.try_create_vault(0, TARGET, DEADLINE),
        "UniversityInactive",
    );
    env.set_university_active(&admin, u, true).unwrap();
    env.create_vault(0);
}

#[test]
fn create_with_inactive_attestor_rejected() {
    let mut env = Env::new();
    let (admin, a) = (env.admin.insecure_clone(), key(&env.attestor));
    env.set_attestor_active(&admin, a, false).unwrap();
    assert_rejected(
        env.try_create_vault(0, TARGET, DEADLINE),
        "AttestorInactive",
    );
}

#[test]
fn registered_parties_must_still_be_distinct() {
    let mut env = Env::new();
    let admin = env.admin.insecure_clone();
    let s = key(&env.student);
    env.register_attestor(&admin, s).unwrap();
    let terms = Terms {
        attestor: s,
        ..env.terms()
    };
    let accounts = env.create_accounts(0, &terms);
    assert_rejected(env.send_create(0, terms, accounts), "InvalidParties");
}

#[test]
fn create_with_mint_other_than_config_mint_rejected() {
    let mut env = Env::new();
    let other = Pubkey::new_unique();
    env.put_mint(other, 6, TOKEN_PROGRAM);
    let terms = env.terms();
    let mut accounts = env.create_accounts(0, &terms);
    accounts.mint = other;
    accounts.vault_token = ata(&accounts.vault, &other);
    assert_rejected(env.send_create(0, terms, accounts), "MintMismatch");
}

/// A vault funded to the target by one funder.
fn funded() -> (Env, Pubkey) {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let a = env.funder(TARGET);
    env.contribute(vault, &a, TARGET).unwrap();
    (env, vault)
}

#[test]
fn confirm_after_attestor_deactivated_rejected() {
    let (mut env, vault) = funded();
    let (admin, attestor) = (env.admin.insecure_clone(), env.attestor.insecure_clone());
    env.set_attestor_active(&admin, key(&attestor), false)
        .unwrap();
    assert_rejected(env.confirm(vault, &attestor), "AttestorInactive");
    assert_eq!(env.balance(&ata(&vault, &env.mint)), TARGET);
}

#[test]
fn confirm_after_university_deactivated_rejected() {
    let (mut env, vault) = funded();
    let (admin, u) = (env.admin.insecure_clone(), key(&env.university));
    env.set_university_active(&admin, u, false).unwrap();
    let attestor = env.attestor.insecure_clone();
    assert_rejected(env.confirm(vault, &attestor), "UniversityInactive");
    assert_eq!(env.balance(&ata(&vault, &env.mint)), TARGET);
}

#[test]
fn confirm_with_registry_entry_of_another_attestor_rejected() {
    let (mut env, vault) = funded();
    let admin = env.admin.insecure_clone();
    let other = Pubkey::new_unique();
    env.register_attestor(&admin, other).unwrap();
    let attestor = env.attestor.insecure_clone();
    let mut accounts = env.confirm_accounts(vault, &attestor, key(&env.university));
    accounts.registered_attestor = attestor_pda(&other);
    assert_rejected(env.send_confirm(accounts, &attestor), "ConstraintSeeds");
}

#[test]
fn contribute_after_university_deactivated_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (admin, u) = (env.admin.insecure_clone(), key(&env.university));
    env.set_university_active(&admin, u, false).unwrap();
    let a = env.funder(10 * USDC);
    assert_rejected(env.contribute(vault, &a, 10 * USDC), "UniversityInactive");
    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 10 * USDC);
}

/// Refunds must never be blockable by the registry.
#[test]
fn deny_expire_cancel_and_refund_work_after_deactivation() {
    let mut env = Env::new();
    let (denied, expired, cancelled) = (
        env.create_vault(0),
        env.create_vault(1),
        env.create_vault(2),
    );
    let a = env.funder(30 * USDC);
    for v in [denied, expired, cancelled] {
        env.contribute(v, &a, 10 * USDC).unwrap();
    }
    let (admin, attestor, student) = (
        env.admin.insecure_clone(),
        env.attestor.insecure_clone(),
        env.student.insecure_clone(),
    );
    let (u, at) = (key(&env.university), key(&attestor));
    env.set_university_active(&admin, u, false).unwrap();
    env.set_attestor_active(&admin, at, false).unwrap();

    env.deny(denied, &attestor).unwrap();
    env.cancel(cancelled, &student).unwrap();
    env.warp(DEADLINE);
    env.expire(expired, &a).unwrap();
    for v in [denied, expired, cancelled] {
        env.refund(v, key(&a), &a).unwrap();
    }
    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 30 * USDC);
}

#[test]
fn contribute_after_attestor_deactivated_rejected() {
    let mut env = Env::new();
    let vault = env.create_vault(0);
    let (admin, at) = (env.admin.insecure_clone(), key(&env.attestor));
    env.set_attestor_active(&admin, at, false).unwrap();
    let a = env.funder(10 * USDC);
    assert_rejected(env.contribute(vault, &a, 10 * USDC), "AttestorInactive");
    assert_eq!(env.balance(&ata(&key(&a), &env.mint)), 10 * USDC);
}
