//! VaultLedger adapter backed by the on-chain Arrhabon program.
#![forbid(unsafe_code)]

pub mod accounts;
pub mod chain;
mod clock;
mod cluster;
mod error;
mod faucet;
pub mod instructions;
mod ledger;
mod relay;
mod signer;

pub use chain::Chain;
pub use clock::ChainClock;
pub use cluster::{Network, ensure_cluster};
pub use faucet::SolanaFaucet;
pub use ledger::SolanaLedger;
pub use signer::{KeypairAttestor, sign_tx};
pub use solana_keypair::{Keypair, read_keypair_file};
