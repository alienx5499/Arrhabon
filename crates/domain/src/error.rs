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
    /// Contribution larger than what is left to reach the target.
    ExceedsTarget,
    /// The vault is not Closed yet.
    NotClosed,
    /// The vault token account still holds tokens.
    BalanceRemaining,
    /// A Contribution account of the vault is still open.
    ContributionsOpen,
}
