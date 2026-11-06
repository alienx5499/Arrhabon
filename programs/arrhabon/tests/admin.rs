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
