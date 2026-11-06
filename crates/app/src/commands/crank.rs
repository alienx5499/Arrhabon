//! Permissionless crank runs: expire due vaults, refund what is owed; reruns are idempotent.

use arrhabon_domain::AccountId;

use crate::ports::{Clock, PortResult, TxSignature, VaultLedger};
use crate::{AppError, PortError};

/// Seconds past the deadline before `ExpireDue` acts. The program checks the deadline
/// against chain time; a local clock running ahead would otherwise send doomed txs.
pub const EXPIRY_GRACE_SECS: i64 = 5;

/// What a crank call achieved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    Submitted(TxSignature),
    /// The chain already showed the effect (an earlier run, or someone else's crank).
    AlreadyApplied,
}

/// Per-item result of a crank run; one failure never stops the others.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<K> {
    pub item: K,
    pub result: Result<Done, AppError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefundKey {
    pub vault: AccountId,
    pub funder: AccountId,
}

fn done(sent: PortResult<TxSignature>) -> Result<Done, AppError> {
    match sent {
        Ok(sig) => Ok(Done::Submitted(sig)),
        Err(PortError::AlreadyApplied) => Ok(Done::AlreadyApplied),
        Err(e) => Err(e.into()),
    }
}

/// Expires every Open vault at least `EXPIRY_GRACE_SECS` past its deadline.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExpireDue;

impl ExpireDue {
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
        clock: &impl Clock,
    ) -> Result<Vec<Outcome<AccountId>>, AppError> {
        let cutoff = clock.now().saturating_sub(EXPIRY_GRACE_SECS);
        let mut outcomes = Vec::new();
        // sequential calls; join them if a crank run gets slow.
        for mut vault in ledger.vaults_due_for_expiry(cutoff).await? {
            if vault.expire(cutoff).is_err() {
                continue; // not due (or no longer Open): nothing to do
            }
            let result = done(ledger.expire(vault.id()).await);
            outcomes.push(Outcome {
                item: vault.id(),
                result,
            });
        }
        Ok(outcomes)
    }
}

/// Refunds every unrefunded contribution of every Refunding vault.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunRefunds;

impl RunRefunds {
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
    ) -> Result<Vec<Outcome<RefundKey>>, AppError> {
        let mut outcomes = Vec::new();
        for c in ledger.pending_refunds().await? {
            if c.refunded() {
                continue;
            }
            let item = RefundKey {
                vault: c.vault(),
                funder: c.funder(),
            };
            let result = done(ledger.refund(item.vault, item.funder).await);
            outcomes.push(Outcome { item, result });
        }
        Ok(outcomes)
    }
}
