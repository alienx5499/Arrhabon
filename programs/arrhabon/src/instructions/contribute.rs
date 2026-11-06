//! `contribute`: any funder adds USDC to an open Vault before its deadline.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use arrhabon_domain::Usdc;

use crate::error::to_anchor;
use crate::events::Contributed;
use crate::state::{CONTRIBUTION_SEED, Contribution, Vault, now};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(mut, has_one = mint)]
    pub vault: Account<'info, Vault>,
    #[account(
        init_if_needed,
        payer = funder,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [CONTRIBUTION_SEED, vault.key().as_ref(), funder.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,
    pub mint: Account<'info, Mint>,
    #[account(mut, token::mint = mint, token::authority = funder)]
    pub funder_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl Contribute<'_> {
    pub fn apply(&mut self, bump: u8, amount: u64) -> Result<()> {
        let key = self.vault.key();
        let funder = self.funder.key();
        let mut vault = self.vault.to_domain(&key)?;
        let mut contribution = self.contribution.to_domain(&key, &funder)?;
        vault
            .contribute(&mut contribution, Usdc::new(amount), now()?)
            .map_err(to_anchor)?;
        self.vault.store(&vault);
        self.contribution.store(&contribution);
        self.contribution.bump = bump;

        let accounts = TransferChecked {
            from: self.funder_token.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.vault_token.to_account_info(),
            authority: self.funder.to_account_info(),
        };
        token::transfer_checked(
            CpiContext::new(self.token_program.key(), accounts),
            amount,
            self.mint.decimals,
        )?;
        emit!(Contributed {
            vault: key,
            funder,
            amount,
            total_contributed: self.vault.total_contributed,
        });
        Ok(())
    }
}
