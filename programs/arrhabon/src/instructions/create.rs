//! `create_vault`: the student opens a Vault and its token account.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use arrhabon_domain::{Deadline, Usdc, Vault as DomainVault};

use crate::error::{ArrhabonError, to_anchor};
use crate::events::VaultCreated;
use crate::state::{
    ATTESTOR_SEED, Attestor, CONFIG_SEED, Config, Status, UNIVERSITY_SEED, University, VAULT_SEED,
    Vault, id, now,
};

#[derive(Accounts)]
#[instruction(nonce: u64, university: Pubkey, attestor: Pubkey)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub student: Signer<'info>,
    #[account(
        init,
        payer = student,
        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_SEED, student.key().as_ref(), &nonce.to_le_bytes()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = mint @ ArrhabonError::MintMismatch)]
    pub config: Account<'info, Config>,
    #[account(
        seeds = [UNIVERSITY_SEED, university.as_ref()],
        bump = registered_university.bump,
        constraint = registered_university.active @ ArrhabonError::UniversityInactive,
    )]
    pub registered_university: Account<'info, University>,
    #[account(
        seeds = [ATTESTOR_SEED, attestor.as_ref()],
        bump = registered_attestor.bump,
        constraint = registered_attestor.active @ ArrhabonError::AttestorInactive,
    )]
    pub registered_attestor: Account<'info, Attestor>,
    /// Decimals were checked when the admin set `config.mint`.
    pub mint: Account<'info, Mint>,
    /// `init_if_needed`: an ATA someone pre-created for this PDA cannot block the nonce.
    #[account(
        init_if_needed,
        payer = student,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl CreateVault<'_> {
    pub fn apply(
        &mut self,
        nonce: u64,
        bump: u8,
        university: Pubkey,
        attestor: Pubkey,
        target: u64,
        deadline: i64,
    ) -> Result<()> {
        let key = self.vault.key();
        let created = DomainVault::create(
            id(&key),
            id(&self.student.key()),
            id(&university),
            id(&attestor),
            Usdc::new(target),
            Deadline::new(deadline),
            now()?,
        )
        .map_err(to_anchor)?;
        self.vault.set_inner(Vault {
            student: self.student.key(),
            university,
            attestor,
            mint: self.mint.key(),
            nonce,
            target,
            deadline,
            status: Status::Open,
            total_contributed: 0,
            refunded_total: 0,
            contributor_count: 0,
            refunded_count: 0,
            has_outside_contributions: false,
            bump,
        });
        self.vault.store(&created);
        emit!(VaultCreated {
            vault: key,
            student: self.student.key(),
            university,
            attestor,
            mint: self.mint.key(),
            target,
            deadline,
        });
        Ok(())
    }
}
