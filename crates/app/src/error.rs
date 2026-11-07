//! Use-case errors: domain rule violations, bad input, missing vaults and port failures.

use arrhabon_domain::{AccountId, DomainError};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PortError {
    /// The chain or remote service refused the request; `code` is the program error
    /// mapped back to its domain rule, `None` for anything else.
    #[error("rejected: {message}")]
    Rejected {
        code: Option<DomainError>,
        message: String,
    },
    /// The chain already shows the effect (same tx processed, vault expired, refund paid).
    #[error("already applied")]
    AlreadyApplied,
    /// The chain or remote service could not be reached.
    #[error("unavailable: {0}")]
    Unavailable(String),
}

impl PortError {
    /// A rejection that carries no program error code.
    pub fn rejected(message: impl Into<String>) -> Self {
        PortError::Rejected {
            code: None,
            message: message.into(),
        }
    }
}

/// Variants are grouped so the API maps each to one status (403 / 409 / 400 / 404 / 503).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppError {
    /// The caller is not the party a rule requires.
    #[error("forbidden: {0:?}")]
    Forbidden(DomainError),
    /// The vault's state or the time does not allow this now.
    #[error("conflict: {0:?}")]
    Conflict(DomainError),
    /// The request itself breaks a rule (amount, parties, deadline range).
    #[error("invalid: {0:?}")]
    Invalid(DomainError),
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("vault not found: {}", crate::address::to_base58(*.0))]
    VaultNotFound(AccountId),
    #[error("university not registered or inactive: {}", crate::address::to_base58(*.0))]
    UniversityNotRegistered(AccountId),
    #[error("attestor not registered or inactive: {}", crate::address::to_base58(*.0))]
    AttestorNotRegistered(AccountId),
    #[error("already applied")]
    AlreadyApplied,
    /// Refused without a program error code (e.g. by the swap service).
    #[error("rejected: {0}")]
    Rejected(String),
    #[error("unavailable: {0}")]
    Unavailable(String),
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        use DomainError::*;
        match e {
            Unauthorized => AppError::Forbidden(e),
            ZeroAmount | Overflow | BelowMinimum | ExceedsTarget | InvalidParties
            | DeadlineTooFar | WrongVault => AppError::Invalid(e),
            Underflow | DeadlinePassed | DeadlineNotReached | NotOpen | NotRefunding
            | NotReleased | BelowTarget | AlreadyRefunded | NothingToRefund
            | ContributionMismatch | CorruptState | NotClosed | BalanceRemaining
            | ContributionsOpen => AppError::Conflict(e),
        }
    }
}

impl From<PortError> for AppError {
    fn from(e: PortError) -> Self {
        match e {
            PortError::Rejected {
                code: Some(code), ..
            } => code.into(),
            PortError::Rejected {
                code: None,
                message,
            } => AppError::Rejected(message),
            PortError::AlreadyApplied => AppError::AlreadyApplied,
            PortError::Unavailable(m) => AppError::Unavailable(m),
        }
    }
}
