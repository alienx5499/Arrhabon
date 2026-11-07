//! One event per Vault state change, for indexers and the API.

use anchor_lang::prelude::*;

use crate::state::Status;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub student: Pubkey,
    pub university: Pubkey,
    pub attestor: Pubkey,
    pub mint: Pubkey,
    pub target: u64,
    pub deadline: i64,
}

#[event]
pub struct Contributed {
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub amount: u64,
    pub total_contributed: u64,
}

#[event]
pub struct Released {
    pub vault: Pubkey,
    pub university: Pubkey,
    pub amount: u64,
}

/// Deny, expiry and cancel: `status` is Refunding, or Closed when nobody contributed.
#[event]
pub struct RefundsStarted {
    pub vault: Pubkey,
    pub reason: RefundReason,
    pub status: Status,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefundReason {
    Denied,
    Expired,
    Cancelled,
}

#[event]
pub struct Refunded {
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub amount: u64,
    pub status: Status,
}

/// Tokens sent to a Closed vault outside `contribute`: to the university after a release,
/// to the student after refunds.
#[event]
pub struct Swept {
    pub vault: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}
