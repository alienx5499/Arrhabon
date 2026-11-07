//! One wrapper per vault instruction, built from Anchor's generated account structs.
//! `*_accounts` builders return the honest accounts so attack tests can swap one field.

use anchor_spl::associated_token;
use anchor_spl::token::ID as TOKEN_PROGRAM;
use arrhabon::accounts;
use arrhabon::instruction as ixn;
use solana_keypair::Keypair;

use super::admin::{attestor_pda, config_pda, university_pda};
use super::{DEADLINE, Env, Pubkey, TARGET, ata, contribution_pda, key, vault_pda};

/// Terms passed to `create_vault`.
#[derive(Clone, Copy)]
pub struct Terms {
    pub university: Pubkey,
    pub attestor: Pubkey,
    pub target: u64,
    pub deadline: i64,
}

impl Env {
    pub fn terms(&self) -> Terms {
        Terms {
            university: key(&self.university),
            attestor: key(&self.attestor),
            target: TARGET,
            deadline: DEADLINE,
        }
    }

    pub fn create_vault(&mut self, nonce: u64) -> Pubkey {
        self.try_create_vault(nonce, TARGET, DEADLINE).unwrap()
    }

    pub fn try_create_vault(
        &mut self,
        nonce: u64,
        target: u64,
        deadline: i64,
    ) -> Result<Pubkey, String> {
        let terms = Terms {
            target,
            deadline,
            ..self.terms()
        };
        let accounts = self.create_accounts(nonce, &terms);
        self.send_create(nonce, terms, accounts)
    }

    pub fn create_accounts(&self, nonce: u64, terms: &Terms) -> accounts::CreateVault {
        let vault = vault_pda(&key(&self.student), nonce);
        accounts::CreateVault {
            student: key(&self.student),
            vault,
            config: config_pda(),
            registered_university: university_pda(&terms.university),
            registered_attestor: attestor_pda(&terms.attestor),
            mint: self.mint,
            vault_token: ata(&vault, &self.mint),
            token_program: TOKEN_PROGRAM,
            associated_token_program: associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
    }

    pub fn send_create(
        &mut self,
        nonce: u64,
        t: Terms,
        accounts: accounts::CreateVault,
    ) -> Result<Pubkey, String> {
        let vault = accounts.vault;
        let data = ixn::CreateVault {
            nonce,
            university: t.university,
            attestor: t.attestor,
            target: t.target,
            deadline: t.deadline,
        };
        let student = self.student.insecure_clone();
        self.send(accounts, data, &[&student]).map(|_| vault)
    }

    pub fn contribute_accounts(&self, vault: Pubkey, funder: &Keypair) -> accounts::Contribute {
        let f = key(funder);
        accounts::Contribute {
            funder: f,
            vault,
            registered_university: university_pda(&key(&self.university)),
            registered_attestor: attestor_pda(&key(&self.attestor)),
            contribution: contribution_pda(&vault, &f),
            mint: self.mint,
            funder_token: ata(&f, &self.mint),
            vault_token: ata(&vault, &self.mint),
            token_program: TOKEN_PROGRAM,
            system_program: anchor_lang::system_program::ID,
        }
    }

    pub fn send_contribute(
        &mut self,
        accounts: accounts::Contribute,
        funder: &Keypair,
        amount: u64,
    ) -> Result<(), String> {
        self.send(accounts, ixn::Contribute { amount }, &[funder])
    }

    /// Contributes from `from` into `vault`, recording into `contribution`.
    pub fn contribute_with(
        &mut self,
        vault: Pubkey,
        funder: &Keypair,
        amount: u64,
        from: Pubkey,
        contribution: Pubkey,
    ) -> Result<(), String> {
        let accounts = accounts::Contribute {
            funder_token: from,
            contribution,
            ..self.contribute_accounts(vault, funder)
        };
        self.send_contribute(accounts, funder, amount)
    }

    pub fn contribute(
        &mut self,
        vault: Pubkey,
        funder: &Keypair,
        amount: u64,
    ) -> Result<(), String> {
        let accounts = self.contribute_accounts(vault, funder);
        self.send_contribute(accounts, funder, amount)
    }

    pub fn confirm_accounts(
        &self,
        vault: Pubkey,
        attestor: &Keypair,
        university: Pubkey,
    ) -> accounts::ConfirmVisa {
        accounts::ConfirmVisa {
            attestor: key(attestor),
            vault,
            university,
            // Registry entries follow the vault's parties, as the program's seeds do.
            registered_university: university_pda(&key(&self.university)),
            registered_attestor: attestor_pda(&key(&self.attestor)),
            mint: self.mint,
            vault_token: ata(&vault, &self.mint),
            university_token: ata(&university, &self.mint),
            token_program: TOKEN_PROGRAM,
            associated_token_program: associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
    }

    pub fn send_confirm(
        &mut self,
        accounts: accounts::ConfirmVisa,
        attestor: &Keypair,
    ) -> Result<(), String> {
        self.send(accounts, ixn::ConfirmVisa {}, &[attestor])
    }

    pub fn confirm_to(
        &mut self,
        vault: Pubkey,
        attestor: &Keypair,
        university: Pubkey,
    ) -> Result<(), String> {
        let accounts = self.confirm_accounts(vault, attestor, university);
        self.send_confirm(accounts, attestor)
    }

    pub fn confirm(&mut self, vault: Pubkey, attestor: &Keypair) -> Result<(), String> {
        let university = key(&self.university);
        self.confirm_to(vault, attestor, university)
    }

    pub fn deny(&mut self, vault: Pubkey, attestor: &Keypair) -> Result<(), String> {
        let accounts = accounts::DenyVisa {
            attestor: key(attestor),
            vault,
        };
        self.send(accounts, ixn::DenyVisa {}, &[attestor])
    }

    pub fn expire(&mut self, vault: Pubkey, caller: &Keypair) -> Result<(), String> {
        self.send(accounts::Expire { vault }, ixn::Expire {}, &[caller])
    }

    pub fn cancel(&mut self, vault: Pubkey, student: &Keypair) -> Result<(), String> {
        let accounts = accounts::Cancel {
            student: key(student),
            vault,
        };
        self.send(accounts, ixn::Cancel {}, &[student])
    }

    pub fn refund_with(
        &mut self,
        vault: Pubkey,
        funder: Pubkey,
        contribution: Pubkey,
        caller: &Keypair,
    ) -> Result<(), String> {
        let accounts = accounts::Refund {
            caller: key(caller),
            vault,
            contribution,
            funder,
            mint: self.mint,
            vault_token: ata(&vault, &self.mint),
            funder_token: ata(&funder, &self.mint),
            token_program: TOKEN_PROGRAM,
            associated_token_program: associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        };
        self.send(accounts, ixn::Refund {}, &[caller])
    }

    pub fn refund(
        &mut self,
        vault: Pubkey,
        funder: Pubkey,
        caller: &Keypair,
    ) -> Result<(), String> {
        self.refund_with(vault, funder, contribution_pda(&vault, &funder), caller)
    }
}
