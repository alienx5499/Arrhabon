//! `confirm_visa` and `deny_visa`: the attestor settles the visa outcome.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use super::pay_from_vault;
use crate::error::to_anchor;
use crate::events::{RefundReason, RefundsStarted, Released};
use crate::state::{Vault, id, now};

#[derive(Accounts)]
pub struct ConfirmVisa<'info> {
    /// Pays for the university's token account if it does not exist yet.
    #[account(mut)]
    pub attestor: Signer<'info>,
    #[account(mut, has_one = mint, has_one = university)]
    pub vault: Account<'info, Vault>,
    /// CHECK: only used as the ATA authority; pinned to `vault.university` by `has_one`.
    pub university: UncheckedAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = attestor,
        associated_token::mint = mint,
        associated_token::authority = university,
        associated_token::token_program = token_program,
    )]
    pub university_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl ConfirmVisa<'_> {
    /// Release: the domain authorises (total >= target), the program pays the token account's
    /// whole balance (including any tokens sent to it directly), then the Vault closes.
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let mut vault = self.vault.to_domain(&key)?;
        vault
            .attest_issued(id(&self.attestor.key()), now()?)
            .map_err(to_anchor)?;
        let amount = self.vault_token.amount;
        pay_from_vault(
            &self.vault,
            &self.vault_token,
            &self.university_token,
            &self.mint,
            &self.token_program,
            amount,
        )?;
        vault.close_released().map_err(to_anchor)?;
        self.vault.store(&vault);
        emit!(Released {
            vault: key,
            university: self.university.key(),
            amount,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct DenyVisa<'info> {
    pub attestor: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}

impl DenyVisa<'_> {
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let mut vault = self.vault.to_domain(&key)?;
        vault
            .attest_denied(id(&self.attestor.key()))
            .map_err(to_anchor)?;
        self.vault.store(&vault);
        emit!(RefundsStarted {
            vault: key,
            reason: RefundReason::Denied,
            status: self.vault.status,
        });
        Ok(())
    }
}
