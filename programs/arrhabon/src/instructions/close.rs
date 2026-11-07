//! After settlement: `sweep` stray tokens to whoever is owed them, then close accounts for rent.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount};
use arrhabon_domain::Usdc;

use super::{as_vault, pay_from_vault};
use crate::error::{ArrhabonError, to_anchor};
use crate::events::Swept;
use crate::state::{CONTRIBUTION_SEED, Contribution, Vault, id};

#[derive(Accounts)]
pub struct Sweep<'info> {
    /// Anyone; pays for the recipient's token account if it does not exist yet.
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(has_one = mint)]
    pub vault: Account<'info, Vault>,
    /// CHECK: only used as the ATA authority; `apply` requires the party the domain names.
    pub recipient: UncheckedAccount<'info>,
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
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Sweep<'_> {
    /// The vault is Closed, so whatever is left was sent outside `contribute`; the domain says
    /// whether the university (after a release) or the student (after refunds) gets it.
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let to = self.vault.to_domain(&key)?.sweep().map_err(to_anchor)?;
        require!(id(&self.recipient.key()) == to, ArrhabonError::Unauthorized);
        let amount = self.vault_token.amount;
        pay_from_vault(
            &self.vault,
            &self.vault_token,
            &self.recipient_token,
            &self.mint,
            &self.token_program,
            amount,
        )?;
        emit!(Swept {
            vault: key,
            recipient: self.recipient.key(),
            amount,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    /// Paid both rents at creation, so gets both back.
    #[account(mut)]
    pub student: Signer<'info>,
    #[account(mut, has_one = mint, has_one = student, close = student)]
    pub vault: Account<'info, Vault>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl CloseVault<'_> {
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let vault = self.vault.to_domain(&key)?;
        vault
            .close(Usdc::new(self.vault_token.amount))
            .map_err(to_anchor)?;
        let accounts = CloseAccount {
            account: self.vault_token.to_account_info(),
            destination: self.student.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        as_vault(&self.vault, |signer| {
            token::close_account(CpiContext::new_with_signer(
                self.token_program.key(),
                accounts,
                signer,
            ))
        })
    }
}

#[derive(Accounts)]
pub struct CloseContribution<'info> {
    /// Counts closed records, so the Vault cannot close while any remain.
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        seeds = [CONTRIBUTION_SEED, vault.key().as_ref(), funder.key().as_ref()],
        bump = contribution.bump,
        close = funder,
    )]
    pub contribution: Account<'info, Contribution>,
    /// CHECK: receives the rent; the contribution PDA seeds bind it to the recorded funder.
    #[account(mut)]
    pub funder: UncheckedAccount<'info>,
}

impl CloseContribution<'_> {
    /// Anyone may close a settled record; its rent always goes to the funder who paid it.
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let mut vault = self.vault.to_domain(&key)?;
        let contribution = self.contribution.to_domain(&key, &self.funder.key())?;
        vault.close_contribution(&contribution).map_err(to_anchor)?;
        self.vault.store(&vault);
        Ok(())
    }
}
