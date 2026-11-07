//! After settlement: sweeping stray tokens and closing accounts to return their rent.

use super::{Contribution, Vault, VaultStatus};
use crate::{AccountId, DomainError, Usdc};

impl Vault {
    /// Who receives tokens left in a Closed vault (sent outside `contribute`): the university
    /// after a release, as `confirm_visa` would have paid them; the student after refunds.
    /// Without this a single stray unit would block `close` forever.
    pub fn sweep(&self) -> Result<AccountId, DomainError> {
        if self.status != VaultStatus::Closed {
            return Err(DomainError::NotClosed);
        }
        // Closed with funders but no refunds is only reachable through a release.
        let released = self.refunded_count == 0 && self.contributor_count > 0;
        Ok(if released {
            self.university
        } else {
            self.student
        })
    }

    /// A Contribution account may close once it is refunded or the vault is Closed.
    pub fn close_contribution(&mut self, contribution: &Contribution) -> Result<(), DomainError> {
        self.ensure_own(contribution)?;
        if !contribution.refunded && self.status != VaultStatus::Closed {
            return Err(DomainError::NotClosed);
        }
        self.closed_contributions = self
            .closed_contributions
            .checked_add(1)
            .ok_or(DomainError::Overflow)?;
        Ok(())
    }

    /// The Vault account may close once Closed, empty and with every Contribution account
    /// closed, so a vault later re-created at the same address never meets old records.
    pub fn close(&self, balance: Usdc) -> Result<(), DomainError> {
        if self.status != VaultStatus::Closed {
            return Err(DomainError::NotClosed);
        }
        if !balance.is_zero() {
            return Err(DomainError::BalanceRemaining);
        }
        match self.closed_contributions == self.contributor_count {
            true => Ok(()),
            false => Err(DomainError::ContributionsOpen),
        }
    }
}
