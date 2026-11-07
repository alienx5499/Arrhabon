//! Program errors: one code per domain rejection, so clients see the exact rule.

use anchor_lang::prelude::*;
use arrhabon_domain::DomainError;

#[error_code]
pub enum ArrhabonError {
    ZeroAmount,
    Overflow,
    Underflow,
    DeadlinePassed,
    DeadlineNotReached,
    NotOpen,
    NotRefunding,
    NotReleased,
    BelowTarget,
    Unauthorized,
    AlreadyRefunded,
    NothingToRefund,
    ContributionMismatch,
    WrongVault,
    InvalidParties,
    CorruptState,
    #[msg("Mint must have 6 decimals")]
    WrongDecimals,
    DeadlineTooFar,
    BelowMinimum,
    #[msg("Mint is not the configured USDC mint")]
    MintMismatch,
    UniversityInactive,
    AttestorInactive,
    ExceedsTarget,
    NotClosed,
    BalanceRemaining,
    ContributionsOpen,
}

pub fn to_anchor(e: DomainError) -> Error {
    match e {
        DomainError::ZeroAmount => ArrhabonError::ZeroAmount,
        DomainError::Overflow => ArrhabonError::Overflow,
        DomainError::Underflow => ArrhabonError::Underflow,
        DomainError::DeadlinePassed => ArrhabonError::DeadlinePassed,
        DomainError::DeadlineNotReached => ArrhabonError::DeadlineNotReached,
        DomainError::NotOpen => ArrhabonError::NotOpen,
        DomainError::NotRefunding => ArrhabonError::NotRefunding,
        DomainError::NotReleased => ArrhabonError::NotReleased,
        DomainError::BelowTarget => ArrhabonError::BelowTarget,
        DomainError::Unauthorized => ArrhabonError::Unauthorized,
        DomainError::AlreadyRefunded => ArrhabonError::AlreadyRefunded,
        DomainError::NothingToRefund => ArrhabonError::NothingToRefund,
        DomainError::ContributionMismatch => ArrhabonError::ContributionMismatch,
        DomainError::WrongVault => ArrhabonError::WrongVault,
        DomainError::InvalidParties => ArrhabonError::InvalidParties,
        DomainError::CorruptState => ArrhabonError::CorruptState,
        DomainError::DeadlineTooFar => ArrhabonError::DeadlineTooFar,
        DomainError::BelowMinimum => ArrhabonError::BelowMinimum,
        DomainError::ExceedsTarget => ArrhabonError::ExceedsTarget,
        DomainError::NotClosed => ArrhabonError::NotClosed,
        DomainError::BalanceRemaining => ArrhabonError::BalanceRemaining,
        DomainError::ContributionsOpen => ArrhabonError::ContributionsOpen,
    }
    .into()
}
