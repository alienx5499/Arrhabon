//! `expire` (anyone, after the deadline) and `cancel` (student, no outside money): start refunds.

use anchor_lang::prelude::*;

use crate::error::to_anchor;
use crate::events::{RefundReason, RefundsStarted};
use crate::state::{Vault, id, now};

#[derive(Accounts)]
pub struct Expire<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}

impl Expire<'_> {
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let mut vault = self.vault.to_domain(&key)?;
        vault.expire(now()?).map_err(to_anchor)?;
        self.vault.store(&vault);
        emit!(RefundsStarted {
            vault: key,
            reason: RefundReason::Expired,
            status: self.vault.status,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub student: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}

impl Cancel<'_> {
    pub fn apply(&mut self) -> Result<()> {
        let key = self.vault.key();
        let mut vault = self.vault.to_domain(&key)?;
        vault.cancel(id(&self.student.key())).map_err(to_anchor)?;
        self.vault.store(&vault);
        emit!(RefundsStarted {
            vault: key,
            reason: RefundReason::Cancelled,
            status: self.vault.status,
        });
        Ok(())
    }
}
