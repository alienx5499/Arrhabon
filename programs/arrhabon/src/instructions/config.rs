//! `init_config`: the program's upgrade authority sets the admin and the USDC mint, once.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_spl::token::Mint;

use crate::error::ArrhabonError;
use crate::state::{CONFIG_SEED, Config, MINT_DECIMALS};

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + Config::INIT_SPACE, seeds = [CONFIG_SEED], bump)]
    pub config: Account<'info, Config>,
    /// Legacy SPL Token mint only (`Account<Mint>` checks the owner).
    #[account(constraint = mint.decimals == MINT_DECIMALS @ ArrhabonError::WrongDecimals)]
    pub mint: Account<'info, Mint>,
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(admin.key()) @ ArrhabonError::Unauthorized,
    )]
    pub program_data: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
}

impl InitConfig<'_> {
    pub fn apply(&mut self, bump: u8) -> Result<()> {
        self.config.set_inner(Config {
            admin: self.admin.key(),
            mint: self.mint.key(),
            bump,
        });
        Ok(())
    }
}
