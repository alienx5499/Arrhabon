//! LiteSVM harness: loads the built program, fakes a 6-decimal mint, wraps each instruction.
#![allow(dead_code)]

mod admin;
mod close;
mod ix;

#[allow(unused_imports)]
pub use admin::{attestor_pda, config_pda, university_pda};
#[allow(unused_imports)]
pub use ix::Terms;

use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token::ID as TOKEN_PROGRAM;
use arrhabon::state::{CONTRIBUTION_SEED, Contribution, VAULT_SEED, Vault};
use litesvm::LiteSVM;
use solana_account::Account;
use solana_address::Address;
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
pub use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::path::Path;
use std::time::SystemTime;

pub use anchor_lang::prelude::Pubkey;

pub const START: i64 = 1_000_000;
pub const DEADLINE: i64 = START + 3_600;
/// One whole USDC in micro-units (also the minimum contribution).
pub const USDC: u64 = 1_000_000;
pub const TARGET: u64 = 1_000 * USDC;
const SO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../target/deploy/arrhabon.so"
);
/// Sources the built program is compiled from; the .so must be newer than all of them.
const SOURCES: [&str; 2] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/src"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/domain/src"),
];
const TOKEN_ACCOUNT_LEN: usize = 165;
/// Upgradeable-loader ProgramData header: tag (4), slot (8), Option<authority> (1 + 32).
const PROGRAMDATA_AUTHORITY: std::ops::Range<usize> = 12..45;

pub struct Env {
    pub svm: LiteSVM,
    pub mint: Pubkey,
    pub student: Keypair,
    pub university: Keypair,
    pub attestor: Keypair,
    /// The program's upgrade authority, and the Config admin after `Env::new`.
    pub admin: Keypair,
}

fn addr(p: &Pubkey) -> Address {
    Address::new_from_array(p.to_bytes())
}

pub fn key(k: &Keypair) -> Pubkey {
    Pubkey::new_from_array(k.pubkey().to_bytes())
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &TOKEN_PROGRAM)
}

pub fn vault_pda(student: &Pubkey, nonce: u64) -> Pubkey {
    let seeds: &[&[u8]] = &[VAULT_SEED, student.as_ref(), &nonce.to_le_bytes()];
    Pubkey::find_program_address(seeds, &arrhabon::ID).0
}

pub fn contribution_pda(vault: &Pubkey, funder: &Pubkey) -> Pubkey {
    let seeds: &[&[u8]] = &[CONTRIBUTION_SEED, vault.as_ref(), funder.as_ref()];
    Pubkey::find_program_address(seeds, &arrhabon::ID).0
}

pub fn program_data() -> Pubkey {
    let seeds: &[&[u8]] = &[arrhabon::ID.as_ref()];
    Pubkey::find_program_address(seeds, &bpf_loader_upgradeable::ID).0
}

fn modified(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .expect("readable file")
}

/// Newest modification time of any file under `dir`.
fn newest(dir: &Path) -> SystemTime {
    let entries = std::fs::read_dir(dir).expect("source directory");
    entries
        .map(|e| e.expect("directory entry").path())
        .map(|p| if p.is_dir() { newest(&p) } else { modified(&p) })
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Tests run the built .so, so a source edit without `anchor build` would test old code.
fn assert_fresh_build() {
    assert!(
        Path::new(SO).exists(),
        "build first: anchor build (target/deploy/arrhabon.so)"
    );
    let newest_source = SOURCES.iter().map(|d| newest(Path::new(d))).max().unwrap();
    assert!(
        modified(Path::new(SO)) >= newest_source,
        "target/deploy/arrhabon.so is older than programs/arrhabon/src or crates/domain/src: \
         run anchor build first (scripts/test.sh does both)"
    );
}

impl Env {
    /// Config initialised by the admin; university and attestor registered and active.
    pub fn new() -> Self {
        let mut env = Self::bare();
        let (admin, mint) = (env.admin.insecure_clone(), env.mint);
        env.init_config(&admin, mint).unwrap();
        let (u, a) = (key(&env.university), key(&env.attestor));
        env.register_university(&admin, u).unwrap();
        env.register_attestor(&admin, a).unwrap();
        env
    }

    /// Program deployed with `admin` as upgrade authority; no Config, no registry.
    pub fn bare() -> Self {
        assert_fresh_build();
        let mut svm = LiteSVM::new();
        svm.add_program_from_file(addr(&arrhabon::ID), SO)
            .expect("build first: anchor build (target/deploy/arrhabon.so)");
        let mut env = Env {
            svm,
            mint: Pubkey::new_unique(),
            student: Keypair::new(),
            university: Keypair::new(),
            attestor: Keypair::new(),
            admin: Keypair::new(),
        };
        env.warp(START);
        env.set_upgrade_authority(key(&env.admin.insecure_clone()));
        env.put_mint(env.mint, 6, TOKEN_PROGRAM);
        let parties = [&env.student, &env.university, &env.attestor, &env.admin].map(key);
        for k in parties {
            env.airdrop(&k);
        }
        env
    }

    fn set_upgrade_authority(&mut self, authority: Pubkey) {
        let at = addr(&program_data());
        let mut account = self.svm.get_account(&at).expect("programdata");
        account.data[PROGRAMDATA_AUTHORITY.start] = 1;
        account.data[PROGRAMDATA_AUTHORITY.start + 1..PROGRAMDATA_AUTHORITY.end]
            .copy_from_slice(authority.as_ref());
        self.svm.set_account(at, account).unwrap();
    }

    /// An initialised mint owned by `token_program`.
    pub fn put_mint(&mut self, at: Pubkey, decimals: u8, token_program: Pubkey) {
        let mut mint = vec![0u8; 82];
        mint[44] = decimals;
        mint[45] = 1; // is_initialized
        self.put_owned(at, mint, token_program);
    }

    pub fn airdrop(&mut self, k: &Pubkey) {
        self.svm.airdrop(&addr(k), 10_000_000_000).unwrap();
    }

    /// A funded signer with `amount` tokens in its ATA.
    pub fn funder(&mut self, amount: u64) -> Keypair {
        let kp = Keypair::new();
        self.airdrop(&key(&kp));
        let owner = key(&kp);
        self.token_account(ata(&owner, &self.mint), &owner, amount);
        kp
    }

    pub fn token_account(&mut self, at: Pubkey, owner: &Pubkey, amount: u64) {
        let mint = self.mint;
        self.token_account_of(at, &mint, owner, amount);
    }

    pub fn token_account_of(&mut self, at: Pubkey, mint: &Pubkey, owner: &Pubkey, amount: u64) {
        let mut data = vec![0u8; TOKEN_ACCOUNT_LEN];
        data[..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[64..72].copy_from_slice(&amount.to_le_bytes());
        data[108] = 1; // AccountState::Initialized
        self.put(at, data);
    }

    fn put(&mut self, at: Pubkey, data: Vec<u8>) {
        self.put_owned(at, data, TOKEN_PROGRAM);
    }

    fn put_owned(&mut self, at: Pubkey, data: Vec<u8>, owner: Pubkey) {
        let lamports = self.svm.minimum_balance_for_rent_exemption(data.len());
        let account = Account {
            lamports,
            data,
            owner: addr(&owner),
            executable: false,
            rent_epoch: 0,
        };
        self.svm.set_account(addr(&at), account).unwrap();
    }

    pub fn warp(&mut self, unix: i64) {
        let mut clock: Clock = self.svm.get_sysvar();
        clock.unix_timestamp = unix;
        self.svm.set_sysvar(&clock);
    }

    pub fn exists(&self, at: &Pubkey) -> bool {
        self.svm
            .get_account(&addr(at))
            .is_some_and(|a| a.lamports > 0)
    }

    pub fn lamports(&self, at: &Pubkey) -> u64 {
        self.svm.get_account(&addr(at)).map_or(0, |a| a.lamports)
    }

    pub fn balance(&self, at: &Pubkey) -> u64 {
        let account = self.svm.get_account(&addr(at)).expect("token account");
        u64::from_le_bytes(account.data[64..72].try_into().unwrap())
    }

    /// Deserialises a program account (checks its discriminator).
    pub fn read<T: AccountDeserialize>(&self, at: &Pubkey) -> T {
        let account = self.svm.get_account(&addr(at)).expect("account");
        T::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    pub fn vault(&self, vault: &Pubkey) -> Vault {
        self.read(vault)
    }

    pub fn contribution(&self, vault: &Pubkey, funder: &Pubkey) -> Contribution {
        self.read(&contribution_pda(vault, funder))
    }

    /// Sends one instruction; the first signer pays. Logs on failure so errors can be matched.
    pub fn send(
        &mut self,
        accounts: impl ToAccountMetas,
        data: impl InstructionData,
        signers: &[&Keypair],
    ) -> Result<(), String> {
        let metas = accounts
            .to_account_metas(None)
            .into_iter()
            .map(|m| AccountMeta {
                pubkey: Address::new_from_array(m.pubkey.to_bytes()),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect();
        let ix = Instruction::new_with_bytes(addr(&arrhabon::ID), &data.data(), metas);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&signers[0].pubkey()),
            signers,
            self.svm.latest_blockhash(),
        );
        let result = self.svm.send_transaction(tx);
        self.svm.expire_blockhash();
        result
            .map(|_| ())
            .map_err(|f| format!("{:?}\n{}", f.err, f.meta.logs.join("\n")))
    }
}

/// Asserts the transaction failed with exactly the Anchor or program error named `code`.
pub fn assert_rejected<T>(result: Result<T, String>, code: &str) {
    let exact = format!("Error Code: {code}. Error Number:");
    match result {
        Ok(_) => panic!("expected rejection with {code}, but it succeeded"),
        Err(logs) => assert!(logs.contains(&exact), "expected {code}, got:\n{logs}"),
    }
}
