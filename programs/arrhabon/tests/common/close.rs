//! Wrappers for the after-settlement instructions: sweep and account closes.

use anchor_spl::associated_token;
use anchor_spl::token::ID as TOKEN_PROGRAM;
use arrhabon::accounts;
use arrhabon::instruction as ixn;
use solana_keypair::Keypair;

use super::{Env, Pubkey, ata, contribution_pda, key};

impl Env {
    /// `recipient` must be the party the program pays strays to (university or student).
    pub fn sweep_accounts(
        &self,
        vault: Pubkey,
        caller: &Keypair,
        recipient: Pubkey,
    ) -> accounts::Sweep {
        accounts::Sweep {
            caller: key(caller),
            vault,
            recipient,
            mint: self.mint,
            vault_token: ata(&vault, &self.mint),
            recipient_token: ata(&recipient, &self.mint),
            token_program: TOKEN_PROGRAM,
            associated_token_program: associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
    }

    pub fn send_sweep(
        &mut self,
        accounts: accounts::Sweep,
        caller: &Keypair,
    ) -> Result<(), String> {
        self.send(accounts, ixn::Sweep {}, &[caller])
    }

    pub fn sweep(&mut self, vault: Pubkey, caller: &Keypair, to: Pubkey) -> Result<(), String> {
        let accounts = self.sweep_accounts(vault, caller, to);
        self.send_sweep(accounts, caller)
    }

    pub fn close_vault(&mut self, vault: Pubkey, student: &Keypair) -> Result<(), String> {
        let accounts = accounts::CloseVault {
            student: key(student),
            vault,
            mint: self.mint,
            vault_token: ata(&vault, &self.mint),
            token_program: TOKEN_PROGRAM,
        };
        self.send(accounts, ixn::CloseVault {}, &[student])
    }

    /// Closes `contribution`, paying its rent to `funder`; `caller` only pays the fee.
    pub fn close_contribution_with(
        &mut self,
        vault: Pubkey,
        funder: Pubkey,
        contribution: Pubkey,
        caller: &Keypair,
    ) -> Result<(), String> {
        let accounts = accounts::CloseContribution {
            vault,
            contribution,
            funder,
        };
        self.send(accounts, ixn::CloseContribution {}, &[caller])
    }

    pub fn close_contribution(
        &mut self,
        vault: Pubkey,
        funder: Pubkey,
        caller: &Keypair,
    ) -> Result<(), String> {
        self.close_contribution_with(vault, funder, contribution_pda(&vault, &funder), caller)
    }
}
