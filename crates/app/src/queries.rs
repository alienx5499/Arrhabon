//! Read models for the student, funder and university screens; they never change state.

use arrhabon_domain::{AccountId, Vault, VaultStatus};

use crate::AppError;
use crate::address::to_base58;
use crate::commands::load_vault;
use crate::ports::{Clock, VaultLedger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributorView {
    pub funder: String,
    pub amount: u64,
    pub refunded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultView {
    pub vault: String,
    pub student: String,
    pub university: String,
    /// The only party that can release the funds.
    pub attestor: String,
    pub status: VaultStatus,
    /// USDC micro-units.
    pub target: u64,
    pub total_contributed: u64,
    pub deadline: i64,
    /// Seconds until the deadline, zero once it has passed.
    pub seconds_left: i64,
    pub contributors: Vec<ContributorView>,
}

impl VaultView {
    pub async fn load(
        ledger: &impl VaultLedger,
        clock: &impl Clock,
        id: AccountId,
    ) -> Result<Self, AppError> {
        Self::from_vault(ledger, clock, load_vault(ledger, id).await?).await
    }

    /// Every vault the student created, with contributors.
    pub async fn for_student(
        ledger: &impl VaultLedger,
        clock: &impl Clock,
        student: AccountId,
    ) -> Result<Vec<Self>, AppError> {
        let mut views = Vec::new();
        for v in ledger.vaults_by_student(student).await? {
            views.push(Self::from_vault(ledger, clock, v).await?);
        }
        Ok(views)
    }

    async fn from_vault(
        ledger: &impl VaultLedger,
        clock: &impl Clock,
        v: Vault,
    ) -> Result<Self, AppError> {
        let id = v.id();
        let contributors = ledger
            .contributions(id)
            .await?
            .iter()
            .map(|c| ContributorView {
                funder: to_base58(c.funder()),
                amount: c.amount().micro(),
                refunded: c.refunded(),
            })
            .collect();
        Ok(VaultView {
            vault: to_base58(v.id()),
            student: to_base58(v.student()),
            university: to_base58(v.university()),
            attestor: to_base58(v.attestor()),
            status: v.status(),
            target: v.target().micro(),
            total_contributed: v.total_contributed().micro(),
            deadline: v.deadline().unix(),
            seconds_left: v.deadline().unix().saturating_sub(clock.now()).max(0),
            contributors,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefundState {
    /// The vault is Open, released or closed without a refund owed.
    NotDue,
    Pending,
    Refunded,
    /// The vault account could not be found.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunderContributionView {
    pub vault: String,
    /// `None` when the vault account could not be found.
    pub vault_status: Option<VaultStatus>,
    pub amount: u64,
    pub refund: RefundState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunderView {
    pub funder: String,
    pub contributions: Vec<FunderContributionView>,
}

impl FunderView {
    pub async fn load(ledger: &impl VaultLedger, funder: AccountId) -> Result<Self, AppError> {
        let mut contributions = Vec::new();
        for c in ledger.contributions_by_funder(funder).await? {
            let vault_status = ledger.vault(c.vault()).await?.map(|v| v.status());
            let refund = match (c.refunded(), vault_status) {
                (true, _) => RefundState::Refunded,
                (false, None) => RefundState::Unavailable,
                (false, Some(VaultStatus::Refunding)) => RefundState::Pending,
                (false, Some(_)) => RefundState::NotDue,
            };
            contributions.push(FunderContributionView {
                vault: to_base58(c.vault()),
                vault_status,
                amount: c.amount().micro(),
                refund,
            });
        }
        Ok(FunderView {
            funder: to_base58(funder),
            contributions,
        })
    }
}

/// What a university can verify before admitting: money held and who can release it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniversityProof {
    pub vault: String,
    pub student: String,
    pub status: VaultStatus,
    /// USDC micro-units still in the vault.
    pub funds_held: u64,
    pub target: u64,
    pub deadline: i64,
    /// The only party that can release the funds, by attesting the visa was issued.
    pub attestor: String,
    pub release_condition: String,
}

impl UniversityProof {
    pub async fn for_vault(ledger: &impl VaultLedger, id: AccountId) -> Result<Self, AppError> {
        Ok(Self::from_vault(&load_vault(ledger, id).await?))
    }

    pub async fn for_university(
        ledger: &impl VaultLedger,
        university: AccountId,
    ) -> Result<Vec<Self>, AppError> {
        let vaults = ledger.vaults_by_university(university).await?;
        Ok(vaults.iter().map(Self::from_vault).collect())
    }

    fn from_vault(v: &Vault) -> Self {
        let release_condition = format!(
            "attestor {} confirms the visa before unix {} with at least {} micro-USDC held; \
             otherwise every funder is refunded",
            to_base58(v.attestor()),
            v.deadline().unix(),
            v.target().micro()
        );
        UniversityProof {
            vault: to_base58(v.id()),
            student: to_base58(v.student()),
            status: v.status(),
            funds_held: v.funds_held().micro(),
            target: v.target().micro(),
            deadline: v.deadline().unix(),
            attestor: to_base58(v.attestor()),
            release_condition,
        }
    }
}

/// Active universities and attestors a student can pick when creating a vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryView {
    pub universities: Vec<String>,
    pub attestors: Vec<String>,
}

impl RegistryView {
    pub async fn load(ledger: &impl VaultLedger) -> Result<Self, AppError> {
        let b58 = |ids: Vec<AccountId>| ids.into_iter().map(to_base58).collect();
        Ok(RegistryView {
            universities: b58(ledger.registered_universities().await?),
            attestors: b58(ledger.registered_attestors().await?),
        })
    }
}
