//! Every way a domain rule can reject a transition.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
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
    OutsideContributions,
    ContributionMismatch,
    /// The Contribution belongs to a different vault.
    WrongVault,
    /// A zero account, or two roles held by the same account.
    InvalidParties,
    /// Stored state that no sequence of transitions could have produced.
    CorruptState,
    /// Deadline more than `MAX_VAULT_DURATION_SECS` after creation.
    DeadlineTooFar,
    /// Contribution smaller than `MIN_CONTRIBUTION`.
    BelowMinimum,
}
