//! Commands: validate with domain transitions on loaded state, then build or submit.
//!
//! The chain is authoritative; the domain pre-checks only fail early with a clear error.

mod crank;
mod faucet;
mod funding;

pub use crank::{Done, EXPIRY_GRACE_SECS, ExpireDue, Outcome, RefundKey, RunRefunds};
pub use faucet::{ClaimTestFunds, Grant, LAMPORTS_GRANT, USDC_GRANT};
pub use funding::{MAX_MIRROR_USDC, Mirror, MirrorSettledSwap, QuoteCrossChainFunding, SwapRecord};

use arrhabon_domain::{AccountId, Contribution, Deadline, Usdc, Vault};

use crate::AppError;
use crate::ports::{
    AttestorSigner, BuiltVault, Clock, NewVault, TxSignature, UnsignedTx, VaultLedger, VisaOutcome,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateVault {
    pub student: AccountId,
    pub university: AccountId,
    pub attestor: AccountId,
    pub target: Usdc,
    pub deadline: Deadline,
    pub nonce: u64,
}

impl CreateVault {
    /// Returns the vault address and the transaction for the student's wallet to sign.
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
        clock: &impl Clock,
    ) -> Result<BuiltVault, AppError> {
        Vault::create(
            ledger.vault_address(self.student, self.nonce),
            self.student,
            self.university,
            self.attestor,
            self.target,
            self.deadline,
            clock.now(),
        )?;
        if !ledger.is_registered_university(self.university).await? {
            return Err(AppError::UniversityNotRegistered(self.university));
        }
        if !ledger.is_registered_attestor(self.attestor).await? {
            return Err(AppError::AttestorNotRegistered(self.attestor));
        }
        Ok(ledger
            .build_create_vault(NewVault {
                student: self.student,
                university: self.university,
                attestor: self.attestor,
                target: self.target,
                deadline: self.deadline,
                nonce: self.nonce,
            })
            .await?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contribute {
    pub vault: AccountId,
    pub funder: AccountId,
    pub amount: Usdc,
}

impl Contribute {
    /// Returns the transaction for the funder's wallet to sign.
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
        clock: &impl Clock,
    ) -> Result<UnsignedTx, AppError> {
        check_funder(self.funder)?;
        let vault = load_vault(ledger, self.vault).await?;
        precheck_contribution(ledger, &vault, self.funder, self.amount, clock.now()).await?;
        Ok(ledger
            .build_contribute(self.vault, self.funder, self.amount)
            .await?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttestVisa {
    pub vault: AccountId,
    pub outcome: VisaOutcome,
}

impl AttestVisa {
    /// Signs as the configured attestor and submits.
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
        signer: &impl AttestorSigner,
        clock: &impl Clock,
    ) -> Result<TxSignature, AppError> {
        let mut vault = load_vault(ledger, self.vault).await?;
        let attestor = signer.attestor();
        match self.outcome {
            VisaOutcome::Issued => vault.attest_issued(attestor, clock.now())?,
            VisaOutcome::Denied => vault.attest_denied(attestor)?,
        }
        let tx = ledger
            .build_attestation(self.vault, attestor, self.outcome)
            .await?;
        let signed = signer.sign(tx).await?;
        Ok(ledger.submit(signed).await?)
    }
}

pub(crate) async fn load_vault(
    ledger: &impl VaultLedger,
    id: AccountId,
) -> Result<Vault, AppError> {
    ledger.vault(id).await?.ok_or(AppError::VaultNotFound(id))
}

fn check_funder(funder: AccountId) -> Result<(), AppError> {
    match funder.is_zero() {
        true => Err(AppError::InvalidInput("funder must be a non-zero account")),
        false => Ok(()),
    }
}

/// Runs `contribute` on a copy of the loaded vault and the funder's stored Contribution.
async fn precheck_contribution(
    ledger: &impl VaultLedger,
    vault: &Vault,
    funder: AccountId,
    amount: Usdc,
    now: i64,
) -> Result<(), AppError> {
    let mut own = ledger
        .contribution(vault.id(), funder)
        .await?
        .unwrap_or_else(|| Contribution::new(vault.id(), funder));
    vault.clone().contribute(&mut own, amount, now)?;
    Ok(())
}
