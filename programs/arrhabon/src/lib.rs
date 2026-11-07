//! Arrhabon on-chain program: adapts accounts to the escrow domain.
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("6p3YF6s88hUk6t2mtNQ792gDBkMUxQxYRTCYafqGj8JE");

#[program]
pub mod arrhabon {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        let bump = ctx.bumps.config;
        ctx.accounts.apply(bump)
    }

    pub fn register_university(ctx: Context<RegisterUniversity>, wallet: Pubkey) -> Result<()> {
        let bump = ctx.bumps.university;
        ctx.accounts.apply(wallet, bump)
    }

    pub fn register_attestor(ctx: Context<RegisterAttestor>, wallet: Pubkey) -> Result<()> {
        let bump = ctx.bumps.attestor;
        ctx.accounts.apply(wallet, bump)
    }

    pub fn set_university_active(ctx: Context<SetUniversityActive>, active: bool) -> Result<()> {
        ctx.accounts.university.active = active;
        Ok(())
    }

    pub fn set_attestor_active(ctx: Context<SetAttestorActive>, active: bool) -> Result<()> {
        ctx.accounts.attestor.active = active;
        Ok(())
    }

    pub fn create_vault(
        ctx: Context<CreateVault>,
        nonce: u64,
        university: Pubkey,
        attestor: Pubkey,
        target: u64,
        deadline: i64,
    ) -> Result<()> {
        let bump = ctx.bumps.vault;
        ctx.accounts
            .apply(nonce, bump, university, attestor, target, deadline)
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.contribution;
        ctx.accounts.apply(bump, amount)
    }

    pub fn confirm_visa(ctx: Context<ConfirmVisa>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn deny_visa(ctx: Context<DenyVisa>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn expire(ctx: Context<Expire>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn sweep(ctx: Context<Sweep>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        ctx.accounts.apply()
    }

    pub fn close_contribution(ctx: Context<CloseContribution>) -> Result<()> {
        ctx.accounts.apply()
    }
}
