//! Stored accounts and their mapping to the domain Vault and Contribution.

use anchor_lang::prelude::*;
use arrhabon_domain as domain;
use domain::{AccountId, Deadline, Usdc, VaultParts};

use crate::error::to_anchor;

pub const VAULT_SEED: &[u8] = b"vault";
pub const CONTRIBUTION_SEED: &[u8] = b"contrib";
pub const CONFIG_SEED: &[u8] = b"config";
pub const UNIVERSITY_SEED: &[u8] = b"university";
pub const ATTESTOR_SEED: &[u8] = b"attestor";
/// The vault holds one USDC-like mint; `Usdc` is micro-units, so 6 decimals.
pub const MINT_DECIMALS: u8 = 6;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum Status {
    Open,
    Released,
    Refunding,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub student: Pubkey,
    pub university: Pubkey,
    pub attestor: Pubkey,
    pub mint: Pubkey,
    pub nonce: u64,
    pub target: u64,
    pub deadline: i64,
    pub status: Status,
    pub total_contributed: u64,
    pub refunded_total: u64,
    pub contributor_count: u32,
    pub refunded_count: u32,
    pub closed_contributions: u32,
    pub bump: u8,
}

/// Program-wide settings, set once by the program's upgrade authority.
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Onboards universities and attestors.
    pub admin: Pubkey,
    /// The one USDC mint every Vault holds.
    pub mint: Pubkey,
    pub bump: u8,
}

/// Registry entry: a university the admin onboarded, PDA `["university", wallet]`.
#[account]
#[derive(InitSpace)]
pub struct University {
    pub wallet: Pubkey,
    pub active: bool,
    pub bump: u8,
}

/// Registry entry: an attestor the admin onboarded, PDA `["attestor", wallet]`.
#[account]
#[derive(InitSpace)]
pub struct Attestor {
    pub wallet: Pubkey,
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub amount: u64,
    pub refunded: bool,
    pub bump: u8,
}

pub fn id(key: &Pubkey) -> AccountId {
    AccountId(key.to_bytes())
}

pub fn now() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

impl Vault {
    pub fn to_domain(&self, key: &Pubkey) -> Result<domain::Vault> {
        domain::Vault::from_parts(VaultParts {
            id: id(key),
            student: id(&self.student),
            university: id(&self.university),
            attestor: id(&self.attestor),
            target: Usdc::new(self.target),
            deadline: Deadline::new(self.deadline),
            status: self.status.into(),
            total_contributed: Usdc::new(self.total_contributed),
            refunded_total: Usdc::new(self.refunded_total),
            contributor_count: self.contributor_count,
            refunded_count: self.refunded_count,
            closed_contributions: self.closed_contributions,
        })
        .map_err(to_anchor)
    }

    /// Writes back the fields transitions can change; identities are fixed at creation.
    pub fn store(&mut self, v: &domain::Vault) {
        self.status = v.status().into();
        self.total_contributed = v.total_contributed().micro();
        self.refunded_total = v.refunded_total().micro();
        self.contributor_count = v.contributor_count();
        self.refunded_count = v.refunded_count();
        self.closed_contributions = v.closed_contributions();
    }
}

impl Contribution {
    /// A freshly initialised account (zero vault) starts as a new domain Contribution.
    pub fn to_domain(&self, vault: &Pubkey, funder: &Pubkey) -> Result<domain::Contribution> {
        if self.vault == Pubkey::default() {
            return Ok(domain::Contribution::new(id(vault), id(funder)));
        }
        domain::Contribution::from_parts(
            id(&self.vault),
            id(&self.funder),
            Usdc::new(self.amount),
            self.refunded,
        )
        .map_err(to_anchor)
    }

    pub fn store(&mut self, c: &domain::Contribution) {
        self.vault = Pubkey::new_from_array(c.vault().0);
        self.funder = Pubkey::new_from_array(c.funder().0);
        self.amount = c.amount().micro();
        self.refunded = c.refunded();
    }
}

impl From<Status> for domain::VaultStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Open => Self::Open,
            Status::Released => Self::Released,
            Status::Refunding => Self::Refunding,
            Status::Closed => Self::Closed,
        }
    }
}

impl From<domain::VaultStatus> for Status {
    fn from(s: domain::VaultStatus) -> Self {
        match s {
            domain::VaultStatus::Open => Self::Open,
            domain::VaultStatus::Released => Self::Released,
            domain::VaultStatus::Refunding => Self::Refunding,
            domain::VaultStatus::Closed => Self::Closed,
        }
    }
}
