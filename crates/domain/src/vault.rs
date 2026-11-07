//! The Vault aggregate and its Contributions: every custody transition and its guards.
//!
//! The Vault holds totals only; each transition takes the one Contribution it touches,
//! so the program can apply it per instruction without loading every funder.

mod close;
mod contribution;
mod restore;

pub use contribution::Contribution;
pub use restore::VaultParts;

use crate::{AccountId, Deadline, DomainError, Usdc};

/// Longest a Vault may stay Open: a deadline at most 365 days after creation.
pub const MAX_VAULT_DURATION_SECS: i64 = 365 * 24 * 60 * 60;
/// Smallest amount per `contribute` call (1 USDC), so dust cannot inflate the funder count.
pub const MIN_CONTRIBUTION: Usdc = Usdc::new(1_000_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultStatus {
    Open,
    Released,
    Refunding,
    Closed,
}

/// Fields are private: state changes only through transitions; `from_parts` loads stored state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vault {
    id: AccountId,
    student: AccountId,
    university: AccountId,
    attestor: AccountId,
    target: Usdc,
    deadline: Deadline,
    status: VaultStatus,
    total_contributed: Usdc,
    refunded_total: Usdc,
    contributor_count: u32,
    refunded_count: u32,
    /// Contribution accounts closed after settlement; the Vault closes only once all are.
    closed_contributions: u32,
}

impl Vault {
    /// The attestor must be independent of student and university (DESIGN: a third party or multisig).
    pub fn create(
        id: AccountId,
        student: AccountId,
        university: AccountId,
        attestor: AccountId,
        target: Usdc,
        deadline: Deadline,
        now: i64,
    ) -> Result<Self, DomainError> {
        check_parties(id, student, university, attestor)?;
        if target.is_zero() {
            return Err(DomainError::ZeroAmount);
        }
        if deadline.is_past(now) {
            return Err(DomainError::DeadlinePassed);
        }
        if deadline.unix() > now.saturating_add(MAX_VAULT_DURATION_SECS) {
            return Err(DomainError::DeadlineTooFar);
        }
        Ok(Vault {
            id,
            student,
            university,
            attestor,
            target,
            deadline,
            status: VaultStatus::Open,
            total_contributed: Usdc::ZERO,
            refunded_total: Usdc::ZERO,
            contributor_count: 0,
            refunded_count: 0,
            closed_contributions: 0,
        })
    }

    pub fn contribute(
        &mut self,
        contribution: &mut Contribution,
        amount: Usdc,
        now: i64,
    ) -> Result<(), DomainError> {
        self.ensure_own(contribution)?;
        self.accepts_contributions(now)?;
        if amount.is_zero() {
            return Err(DomainError::ZeroAmount);
        }
        let remaining = self.target.checked_sub(self.total_contributed)?;
        if amount > remaining {
            return Err(DomainError::ExceedsTarget);
        }
        // A remainder below the minimum may still be paid exactly, so every vault can fill.
        if amount < MIN_CONTRIBUTION && amount != remaining {
            return Err(DomainError::BelowMinimum);
        }
        let total = self.total_contributed.checked_add(amount)?;
        let own = contribution.amount.checked_add(amount)?;
        let count = if contribution.amount.is_zero() {
            self.contributor_count
                .checked_add(1)
                .ok_or(DomainError::Overflow)?
        } else {
            self.contributor_count
        };
        self.total_contributed = total;
        self.contributor_count = count;
        contribution.amount = own;
        Ok(())
    }

    /// Open and before the deadline; amount rules are checked by `contribute`.
    pub fn accepts_contributions(&self, now: i64) -> Result<(), DomainError> {
        self.ensure_open()?;
        match self.deadline.is_past(now) {
            true => Err(DomainError::DeadlinePassed),
            false => Ok(()),
        }
    }

    /// Visa issued: the whole `total_contributed` goes to the university.
    pub fn attest_issued(&mut self, attestor: AccountId, now: i64) -> Result<(), DomainError> {
        self.ensure_attestor(attestor)?;
        self.ensure_open()?;
        if self.deadline.is_past(now) {
            return Err(DomainError::DeadlinePassed);
        }
        if self.total_contributed < self.target {
            return Err(DomainError::BelowTarget);
        }
        self.status = VaultStatus::Released;
        Ok(())
    }

    pub fn attest_denied(&mut self, attestor: AccountId) -> Result<(), DomainError> {
        self.ensure_attestor(attestor)?;
        self.ensure_open()?;
        self.start_refunds();
        Ok(())
    }

    pub fn expire(&mut self, now: i64) -> Result<(), DomainError> {
        self.ensure_open()?;
        if !self.deadline.is_past(now) {
            return Err(DomainError::DeadlineNotReached);
        }
        self.start_refunds();
        Ok(())
    }

    pub fn cancel(&mut self, student: AccountId) -> Result<(), DomainError> {
        if student != self.student {
            return Err(DomainError::Unauthorized);
        }
        self.ensure_open()?;
        self.start_refunds();
        Ok(())
    }

    /// Marks `contribution` refunded; the caller then pays `contribution.amount` to its funder.
    pub fn refund(&mut self, contribution: &mut Contribution) -> Result<(), DomainError> {
        self.ensure_own(contribution)?;
        if contribution.refunded {
            return Err(DomainError::AlreadyRefunded);
        }
        if self.status != VaultStatus::Refunding {
            return Err(DomainError::NotRefunding);
        }
        if contribution.amount.is_zero() {
            return Err(DomainError::NothingToRefund);
        }
        let refunded = self.refunded_total.checked_add(contribution.amount)?;
        if refunded > self.total_contributed {
            return Err(DomainError::ContributionMismatch);
        }
        let count = self
            .refunded_count
            .checked_add(1)
            .ok_or(DomainError::Overflow)?;
        self.refunded_total = refunded;
        self.refunded_count = count;
        contribution.refunded = true;
        if self.refunded_count == self.contributor_count {
            self.status = VaultStatus::Closed;
        }
        Ok(())
    }

    /// Released -> Closed, once the program has paid `total_contributed` to the university.
    pub fn close_released(&mut self) -> Result<(), DomainError> {
        if self.status != VaultStatus::Released {
            return Err(DomainError::NotReleased);
        }
        self.status = VaultStatus::Closed;
        Ok(())
    }

    fn start_refunds(&mut self) {
        self.status = if self.contributor_count == 0 {
            VaultStatus::Closed
        } else {
            VaultStatus::Refunding
        };
    }

    fn ensure_open(&self) -> Result<(), DomainError> {
        match self.status {
            VaultStatus::Open => Ok(()),
            _ => Err(DomainError::NotOpen),
        }
    }

    fn ensure_own(&self, contribution: &Contribution) -> Result<(), DomainError> {
        match contribution.vault == self.id {
            true => Ok(()),
            false => Err(DomainError::WrongVault),
        }
    }

    fn ensure_attestor(&self, attestor: AccountId) -> Result<(), DomainError> {
        match attestor == self.attestor {
            true => Ok(()),
            false => Err(DomainError::Unauthorized),
        }
    }
}

/// Getters: read-only views of the stored state.
impl Vault {
    pub fn id(&self) -> AccountId {
        self.id
    }
    pub fn student(&self) -> AccountId {
        self.student
    }
    pub fn university(&self) -> AccountId {
        self.university
    }
    pub fn attestor(&self) -> AccountId {
        self.attestor
    }
    pub fn target(&self) -> Usdc {
        self.target
    }
    pub fn deadline(&self) -> Deadline {
        self.deadline
    }
    pub fn status(&self) -> VaultStatus {
        self.status
    }
    pub fn total_contributed(&self) -> Usdc {
        self.total_contributed
    }
    pub fn refunded_total(&self) -> Usdc {
        self.refunded_total
    }
    /// USDC still in custody: contributions not yet refunded; nothing once released.
    pub fn funds_held(&self) -> Usdc {
        match self.status {
            VaultStatus::Released | VaultStatus::Closed => Usdc::ZERO,
            VaultStatus::Open | VaultStatus::Refunding => Usdc::new(
                self.total_contributed
                    .micro()
                    .saturating_sub(self.refunded_total.micro()),
            ),
        }
    }
    pub fn contributor_count(&self) -> u32 {
        self.contributor_count
    }
    pub fn refunded_count(&self) -> u32 {
        self.refunded_count
    }
    pub fn closed_contributions(&self) -> u32 {
        self.closed_contributions
    }
}

/// No zero accounts, and student, university and attestor are three different parties.
fn check_parties(
    id: AccountId,
    student: AccountId,
    university: AccountId,
    attestor: AccountId,
) -> Result<(), DomainError> {
    let zero = [id, student, university, attestor]
        .iter()
        .any(|a| a.is_zero());
    if zero || student == university || attestor == student || attestor == university {
        return Err(DomainError::InvalidParties);
    }
    Ok(())
}
