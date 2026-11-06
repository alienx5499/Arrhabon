//! Admin-only registry: onboard universities and attestors and switch them on or off.
//! A student can only name parties the admin registered, never appoint their own.

use anchor_lang::prelude::*;

use crate::error::ArrhabonError;
use crate::state::{ATTESTOR_SEED, Attestor, CONFIG_SEED, Config, UNIVERSITY_SEED, University};

#[derive(Accounts)]
#[instruction(wallet: Pubkey)]
pub struct RegisterUniversity<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin @ ArrhabonError::Unauthorized)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = 8 + University::INIT_SPACE,
        seeds = [UNIVERSITY_SEED, wallet.as_ref()],
        bump,
    )]
    pub university: Account<'info, University>,
    pub system_program: Program<'info, System>,
}

impl RegisterUniversity<'_> {
    pub fn apply(&mut self, wallet: Pubkey, bump: u8) -> Result<()> {
        self.university.set_inner(University {
            wallet,
            active: true,
            bump,
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(wallet: Pubkey)]
pub struct RegisterAttestor<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin @ ArrhabonError::Unauthorized)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = 8 + Attestor::INIT_SPACE,
        seeds = [ATTESTOR_SEED, wallet.as_ref()],
        bump,
    )]
    pub attestor: Account<'info, Attestor>,
    pub system_program: Program<'info, System>,
}

impl RegisterAttestor<'_> {
    pub fn apply(&mut self, wallet: Pubkey, bump: u8) -> Result<()> {
        self.attestor.set_inner(Attestor {
            wallet,
            active: true,
            bump,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetUniversityActive<'info> {
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin @ ArrhabonError::Unauthorized)]
    pub config: Account<'info, Config>,
    #[account(mut, seeds = [UNIVERSITY_SEED, university.wallet.as_ref()], bump = university.bump)]
    pub university: Account<'info, University>,
}

#[derive(Accounts)]
pub struct SetAttestorActive<'info> {
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin @ ArrhabonError::Unauthorized)]
    pub config: Account<'info, Config>,
    #[account(mut, seeds = [ATTESTOR_SEED, attestor.wallet.as_ref()], bump = attestor.bump)]
    pub attestor: Account<'info, Attestor>,
}
