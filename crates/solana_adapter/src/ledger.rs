//! `VaultLedger` on the Arrhabon program: wallet transactions, reads, crank calls.

use std::collections::HashSet;
use std::sync::Arc;

use anchor_client::Signer;
use anchor_client::compat::solana_transaction::Transaction;
use anchor_lang::prelude::Pubkey;
use arrhabon::state::{self, Status, id};
use arrhabon_app::PortError;
use arrhabon_app::ports::{
    BuiltVault, NewVault, PortResult, SignedTx, TxSignature, UnsignedTx, VaultLedger, VisaOutcome,
};
use arrhabon_domain::{AccountId, Contribution, DomainError, Usdc, Vault};
use solana_keypair::Keypair;

use crate::accounts::{
    CONTRIBUTION_FUNDER, CONTRIBUTION_VAULT, VAULT_STUDENT, VAULT_UNIVERSITY, key_filter,
    status_filter, unrefunded_filter,
};
use crate::chain::Chain;
use crate::cluster::Network;
use crate::error::applied_if;
use crate::instructions::{self as ixs, VaultTerms};

fn pk(a: AccountId) -> Pubkey {
    Pubkey::new_from_array(a.0)
}

pub struct SolanaLedger {
    chain: Chain,
    /// Signs and pays for permissionless `expire` and `refund`.
    crank: Arc<Keypair>,
    mint: Pubkey,
}

impl SolanaLedger {
    /// Refuses an RPC the cluster guard rejects, then reads the USDC mint from Config.
    pub async fn connect(
        network: Network,
        rpc_url: &str,
        crank: Keypair,
    ) -> Result<Self, PortError> {
        let crank = Arc::new(crank);
        crate::cluster::ensure_cluster(network, rpc_url).await?;
        let chain = Chain::new(rpc_url, crank.clone())?;
        let config: state::Config = chain
            .fetch(ixs::config_pda())
            .await?
            .ok_or_else(|| PortError::rejected("program Config not initialised"))?;
        Ok(SolanaLedger {
            chain,
            crank,
            mint: config.mint,
        })
    }

    pub fn chain(&self) -> &Chain {
        &self.chain
    }

    pub fn mint(&self) -> Pubkey {
        self.mint
    }

    async fn stored_vault(&self, vault: AccountId) -> PortResult<state::Vault> {
        self.chain
            .fetch(pk(vault))
            .await?
            .ok_or_else(|| PortError::rejected("vault not found"))
    }
}

impl VaultLedger for SolanaLedger {
    fn vault_address(&self, student: AccountId, nonce: u64) -> AccountId {
        id(&ixs::vault_pda(&pk(student), nonce))
    }

    async fn is_registered_university(&self, wallet: AccountId) -> PortResult<bool> {
        let entry: Option<state::University> =
            self.chain.fetch(ixs::university_pda(&pk(wallet))).await?;
        Ok(entry.is_some_and(|u| u.active))
    }

    async fn is_registered_attestor(&self, wallet: AccountId) -> PortResult<bool> {
        let entry: Option<state::Attestor> =
            self.chain.fetch(ixs::attestor_pda(&pk(wallet))).await?;
        Ok(entry.is_some_and(|a| a.active))
    }

    async fn registered_universities(&self) -> PortResult<Vec<AccountId>> {
        self.chain.registered_universities().await
    }

    async fn registered_attestors(&self) -> PortResult<Vec<AccountId>> {
        self.chain.registered_attestors().await
    }

    async fn vault(&self, vault: AccountId) -> PortResult<Option<Vault>> {
        let key = pk(vault);
        let stored: Option<state::Vault> = self.chain.fetch(key).await?;
        stored.map(|v| crate::accounts::vault(&key, &v)).transpose()
    }

    async fn vaults_by_student(&self, student: AccountId) -> PortResult<Vec<Vault>> {
        let filter = key_filter(VAULT_STUDENT, &pk(student));
        self.chain.vaults(vec![filter]).await
    }

    async fn vaults_by_university(&self, university: AccountId) -> PortResult<Vec<Vault>> {
        let filter = key_filter(VAULT_UNIVERSITY, &pk(university));
        self.chain.vaults(vec![filter]).await
    }

    async fn contributions(&self, vault: AccountId) -> PortResult<Vec<Contribution>> {
        let filter = key_filter(CONTRIBUTION_VAULT, &pk(vault));
        self.chain.contributions(vec![filter]).await
    }

    async fn contribution(
        &self,
        vault: AccountId,
        funder: AccountId,
    ) -> PortResult<Option<Contribution>> {
        let key = ixs::contribution_pda(&pk(vault), &pk(funder));
        let stored: Option<state::Contribution> = self.chain.fetch(key).await?;
        stored
            .map(|c| crate::accounts::contribution(&c))
            .transpose()
    }

    async fn contributions_by_funder(&self, funder: AccountId) -> PortResult<Vec<Contribution>> {
        let filter = key_filter(CONTRIBUTION_FUNDER, &pk(funder));
        self.chain.contributions(vec![filter]).await
    }

    /// Every Open vault: deadlines cannot be range-filtered by memcmp.
    async fn vaults_due_for_expiry(&self, _now: i64) -> PortResult<Vec<Vault>> {
        self.chain.vaults(vec![status_filter(Status::Open)]).await
    }

    async fn pending_refunds(&self) -> PortResult<Vec<Contribution>> {
        let refunding: HashSet<AccountId> = self
            .chain
            .vaults(vec![status_filter(Status::Refunding)])
            .await?
            .iter()
            .map(Vault::id)
            .collect();
        if refunding.is_empty() {
            return Ok(Vec::new());
        }
        let unrefunded = self.chain.contributions(vec![unrefunded_filter()]).await?;
        Ok(unrefunded
            .into_iter()
            .filter(|c| refunding.contains(&c.vault()))
            .collect())
    }

    async fn build_create_vault(&self, v: NewVault) -> PortResult<BuiltVault> {
        let student = pk(v.student);
        let terms = VaultTerms {
            student,
            nonce: v.nonce,
            university: pk(v.university),
            attestor: pk(v.attestor),
            target: v.target.micro(),
            deadline: v.deadline.unix(),
        };
        let tx = self
            .chain
            .unsigned(&[ixs::create_vault(terms, self.mint)], &student)
            .await?;
        Ok(BuiltVault {
            vault: self.vault_address(v.student, v.nonce),
            tx,
        })
    }

    async fn build_contribute(
        &self,
        vault: AccountId,
        funder: AccountId,
        amount: Usdc,
    ) -> PortResult<UnsignedTx> {
        let ix = ixs::contribute(pk(vault), pk(funder), self.mint, amount.micro());
        self.chain.unsigned(&[ix], &pk(funder)).await
    }

    async fn build_attestation(
        &self,
        vault: AccountId,
        attestor: AccountId,
        outcome: VisaOutcome,
    ) -> PortResult<UnsignedTx> {
        let ix = match outcome {
            VisaOutcome::Issued => {
                let stored = self.stored_vault(vault).await?;
                ixs::confirm_visa(pk(vault), pk(attestor), stored.university, stored.mint)
            }
            VisaOutcome::Denied => ixs::deny_visa(pk(vault), pk(attestor)),
        };
        self.chain.unsigned(&[ix], &pk(attestor)).await
    }

    async fn submit(&self, tx: SignedTx) -> PortResult<TxSignature> {
        let tx: Transaction = bincode::deserialize(&tx.0)
            .map_err(|_| PortError::rejected("not a serialized legacy transaction"))?;
        if !tx.is_signed() {
            return Err(PortError::rejected("transaction is missing signatures"));
        }
        crate::relay::check_programs(&tx)?;
        self.chain.send(&tx).await
    }

    async fn expire(&self, vault: AccountId) -> PortResult<TxSignature> {
        match self
            .chain
            .send_as(&[ixs::expire(pk(vault))], &self.crank)
            .await
        {
            Err(
                e @ PortError::Rejected {
                    code: Some(DomainError::NotOpen),
                    ..
                },
            ) => {
                let stored: Option<state::Vault> = self.chain.fetch(pk(vault)).await?;
                Err(expiry_error(e, stored.as_ref()))
            }
            other => other,
        }
    }

    async fn refund(&self, vault: AccountId, funder: AccountId) -> PortResult<TxSignature> {
        let ix = ixs::refund(pk(vault), pk(funder), self.crank.pubkey(), self.mint);
        self.chain
            .send_as(&[ix], &self.crank)
            .await
            .map_err(|e| applied_if(e, &[DomainError::AlreadyRefunded]))
    }
}

/// `NotOpen` on expire means "already expired" only when the vault went down the refund
/// path (Refunding, or Closed with nothing contributed). A Released vault, or one Closed
/// after paying out, stays a non-retryable rejection: expiry did not and will not happen.
fn expiry_error(e: PortError, stored: Option<&state::Vault>) -> PortError {
    match stored {
        Some(v) if v.status == Status::Refunding => applied_if(e, &[DomainError::NotOpen]),
        Some(v) if v.status == Status::Closed && v.total_contributed == 0 => {
            applied_if(e, &[DomainError::NotOpen])
        }
        _ => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::stored_vault;

    fn not_open() -> PortError {
        PortError::Rejected {
            code: Some(DomainError::NotOpen),
            message: "NotOpen".into(),
        }
    }

    #[test]
    fn expiry_on_refunding_vault_is_already_applied() {
        let v = stored_vault(Status::Refunding);
        assert_eq!(
            expiry_error(not_open(), Some(&v)),
            PortError::AlreadyApplied
        );
        let empty_closed = stored_vault(Status::Closed);
        assert_eq!(
            expiry_error(not_open(), Some(&empty_closed)),
            PortError::AlreadyApplied
        );
    }

    #[test]
    fn expiry_on_released_or_paid_out_vault_stays_rejected() {
        let released = stored_vault(Status::Released);
        assert_eq!(expiry_error(not_open(), Some(&released)), not_open());
        let mut paid = stored_vault(Status::Closed);
        paid.total_contributed = 1_000_000_000;
        assert_eq!(expiry_error(not_open(), Some(&paid)), not_open());
        assert_eq!(expiry_error(not_open(), None), not_open());
    }
}
