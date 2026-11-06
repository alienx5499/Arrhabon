//! Loading a Vault from stored state, rejecting anything transitions could not produce.

use super::{Vault, VaultStatus, check_parties};
use crate::{AccountId, Deadline, DomainError, Usdc};

/// Every stored field of a Vault, as the program reads it from its account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultParts {
    pub id: AccountId,
    pub student: AccountId,
    pub university: AccountId,
    pub attestor: AccountId,
    pub target: Usdc,
    pub deadline: Deadline,
    pub status: VaultStatus,
    pub total_contributed: Usdc,
    pub refunded_total: Usdc,
    pub contributor_count: u32,
    pub refunded_count: u32,
    pub has_outside_contributions: bool,
}

impl Vault {
    pub fn from_parts(p: VaultParts) -> Result<Self, DomainError> {
        check_parties(p.id, p.student, p.university, p.attestor)?;
        if !is_consistent(&p) {
            return Err(DomainError::CorruptState);
        }
        Ok(Vault {
            id: p.id,
            student: p.student,
            university: p.university,
            attestor: p.attestor,
            target: p.target,
            deadline: p.deadline,
            status: p.status,
            total_contributed: p.total_contributed,
            refunded_total: p.refunded_total,
            contributor_count: p.contributor_count,
            refunded_count: p.refunded_count,
            has_outside_contributions: p.has_outside_contributions,
        })
    }
}

fn is_consistent(p: &VaultParts) -> bool {
    // Every recorded contribution is non-zero, so counts and totals move together.
    let none = p.refunded_count == 0;
    let all = p.refunded_count == p.contributor_count;
    let funded = p.total_contributed >= p.target;
    let shape = !p.target.is_zero()
        && p.deadline.unix() > 0
        && p.refunded_count <= p.contributor_count
        && p.refunded_total <= p.total_contributed
        && (p.contributor_count == 0) == p.total_contributed.is_zero()
        && (p.contributor_count > 0 || !p.has_outside_contributions)
        && none == p.refunded_total.is_zero()
        && all == (p.refunded_total == p.total_contributed);
    let status = match p.status {
        VaultStatus::Open => none,
        VaultStatus::Released => none && funded,
        VaultStatus::Refunding => !all,
        // Refunding-derived: everyone refunded (or nobody funded). Released-derived: no refunds.
        VaultStatus::Closed => all || (none && funded),
    };
    shape && status
}
