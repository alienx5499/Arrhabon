//! The fake chain's accounts and how a signed action changes them.

use std::collections::{BTreeMap, BTreeSet};

use arrhabon_domain::{AccountId, Contribution, DomainError, Usdc, Vault};

use crate::PortError;
use crate::ports::{NewVault, PortResult, TxSignature, VisaOutcome};

#[derive(Debug, Clone)]
pub(super) enum Action {
    Create(NewVault, AccountId),
    Contribute(AccountId, AccountId, Usdc),
    Attest(AccountId, VisaOutcome),
}

pub(super) type Vaults = BTreeMap<AccountId, Vault>;
pub(super) type Contributions = BTreeMap<(AccountId, AccountId), Contribution>;

#[derive(Debug, Default)]
pub(super) struct State {
    pub vaults: Vaults,
    pub contributions: Contributions,
    /// Reads served from this copy instead of live state: a lagging RPC node.
    pub frozen: Option<(Vaults, Contributions)>,
    /// Built transactions; `None` once submitted.
    pub pending: Vec<Option<Action>>,
    pub universities: BTreeSet<AccountId>,
    pub attestors: BTreeSet<AccountId>,
    pub failing: BTreeSet<AccountId>,
    pub without_token_account: BTreeSet<AccountId>,
    sent: u64,
}

pub(super) fn rejected(e: DomainError) -> PortError {
    PortError::Rejected {
        code: Some(e),
        message: format!("{e:?}"),
    }
}

impl State {
    pub fn read(&self) -> (&Vaults, &Contributions) {
        match &self.frozen {
            Some((v, c)) => (v, c),
            None => (&self.vaults, &self.contributions),
        }
    }

    pub fn vault_mut(&mut self, id: AccountId) -> PortResult<&mut Vault> {
        self.vaults
            .get_mut(&id)
            .ok_or_else(|| PortError::rejected("vault account missing"))
    }

    pub fn signature(&mut self) -> TxSignature {
        self.sent += 1;
        TxSignature(format!("sig-{}", self.sent))
    }

    pub fn ensure_not_failing(&self, ids: &[AccountId]) -> PortResult<()> {
        match ids.iter().any(|id| self.failing.contains(id)) {
            true => Err(PortError::Unavailable("injected failure".into())),
            false => Ok(()),
        }
    }

    pub fn apply(&mut self, action: Action, signer: AccountId, now: i64) -> PortResult<()> {
        match action {
            Action::Create(n, id) => {
                if signer != n.student {
                    return Err(PortError::rejected("missing student signature"));
                }
                if !self.universities.contains(&n.university)
                    || !self.attestors.contains(&n.attestor)
                {
                    return Err(PortError::rejected("party not registered"));
                }
                let v = Vault::create(
                    id,
                    n.student,
                    n.university,
                    n.attestor,
                    n.target,
                    n.deadline,
                    now,
                )
                .map_err(rejected)?;
                self.vaults.insert(id, v);
            }
            Action::Contribute(vault, funder, amount) => {
                if signer != funder {
                    return Err(PortError::rejected("missing funder signature"));
                }
                let key = (vault, funder);
                let mut c = self
                    .contributions
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| Contribution::new(vault, funder));
                self.vault_mut(vault)?
                    .contribute(&mut c, amount, now)
                    .map_err(rejected)?;
                self.contributions.insert(key, c);
            }
            Action::Attest(vault, VisaOutcome::Issued) => {
                let v = self.vault_mut(vault)?;
                v.attest_issued(signer, now).map_err(rejected)?;
                v.close_released().map_err(rejected)?;
            }
            Action::Attest(vault, VisaOutcome::Denied) => {
                self.vault_mut(vault)?
                    .attest_denied(signer)
                    .map_err(rejected)?;
            }
        }
        Ok(())
    }
}
