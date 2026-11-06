//! One funder's stake in one vault.

use crate::{AccountId, DomainError, Usdc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    pub(super) vault: AccountId,
    pub(super) funder: AccountId,
    pub(super) amount: Usdc,
    pub(super) refunded: bool,
}

impl Contribution {
    pub fn new(vault: AccountId, funder: AccountId) -> Self {
        Contribution {
            vault,
            funder,
            amount: Usdc::ZERO,
            refunded: false,
        }
    }

    /// Loads stored state; rejects zero accounts and a refund of nothing.
    pub fn from_parts(
        vault: AccountId,
        funder: AccountId,
        amount: Usdc,
        refunded: bool,
    ) -> Result<Self, DomainError> {
        if vault.is_zero() || funder.is_zero() {
            return Err(DomainError::InvalidParties);
        }
        if refunded && amount.is_zero() {
            return Err(DomainError::CorruptState);
        }
        Ok(Contribution {
            vault,
            funder,
            amount,
            refunded,
        })
    }

    pub fn vault(&self) -> AccountId {
        self.vault
    }

    pub fn funder(&self) -> AccountId {
        self.funder
    }

    pub fn amount(&self) -> Usdc {
        self.amount
    }

    pub fn refunded(&self) -> bool {
        self.refunded
    }
}
