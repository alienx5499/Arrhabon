//! Escrow domain: Vault aggregate, money rules and invariants, free of I/O.
#![no_std]
#![forbid(unsafe_code)]

mod deadline;
mod error;
mod usdc;
mod vault;

pub use deadline::Deadline;
pub use error::DomainError;
pub use usdc::Usdc;
pub use vault::{
    Contribution, MAX_VAULT_DURATION_SECS, MIN_CONTRIBUTION, Vault, VaultParts, VaultStatus,
};

/// A 32-byte account identity (a Solana pubkey on chain, kept chain-agnostic here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(pub [u8; 32]);

impl AccountId {
    pub fn is_zero(self) -> bool {
        self.0 == [0; 32]
    }
}
