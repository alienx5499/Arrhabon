//! Instruction handlers: load accounts into the domain, apply one transition, store, pay, emit.

pub mod attest;
pub mod config;
pub mod contribute;
pub mod create;
pub mod refund;
pub mod registry;
pub mod settle;

pub use attest::*;
pub use config::*;
pub use contribute::*;
pub use create::*;
pub use refund::*;
pub use registry::*;
pub use settle::*;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::state::{VAULT_SEED, Vault};

/// Pays `amount` out of the vault token account, signed by the vault PDA.
pub fn pay_from_vault<'info>(
    vault: &Account<'info, Vault>,
    vault_token: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let nonce = vault.nonce.to_le_bytes();
    let seeds: &[&[u8]] = &[VAULT_SEED, vault.student.as_ref(), &nonce, &[vault.bump]];
    let accounts = TransferChecked {
        from: vault_token.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: vault.to_account_info(),
    };
    token::transfer_checked(
        CpiContext::new_with_signer(token_program.key(), accounts, &[seeds]),
        amount,
        mint.decimals,
    )
}
