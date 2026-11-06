//! `refund`: anyone pays one recorded funder back in full once the Vault is Refunding.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use super::pay_from_vault;
use crate::error::to_anchor;
use crate::events::Refunded;
use crate::state::{CONTRIBUTION_SEED, Contribution, Vault};

#[derive(Accounts)]
pub struct Refund<'info> {
    /// Pays for the funder's token account if it does not exist yet.
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(mut, has_one = mint)]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        seeds = [CONTRIBUTION_SEED, vault.key().as_ref(), funder.key().as_ref()],
        bump = contribution.bump,
    )]
    pub contribution: Account<'info, Contribution>,
    /// CHECK: identity only; the contribution PDA seeds bind it to the recorded funder.
    pub funder: UncheckedAccount<'info>,
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
        payer = caller,
        associated_token::mint = mint,
        associated_token::authority = funder,
        associated_token::token_program = token_program,
    )]
    pub funder_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Refund<'_> {
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let funder = self.funder.key();
        let mut vault = self.vault.to_domain(&key)?;
        let mut contribution = self.contribution.to_domain(&key, &funder)?;
        vault.refund(&mut contribution).map_err(to_anchor)?;
        let amount = contribution.amount().micro();
        pay_from_vault(
            &self.vault,
            &self.vault_token,
            &self.funder_token,
            &self.mint,
            &self.token_program,
            amount,
        )?;
        self.vault.store(&vault);
        self.contribution.store(&contribution);
        emit!(Refunded {
            vault: key,
            funder,
            amount,
            status: self.vault.status,
        });
        Ok(())
    }
}
