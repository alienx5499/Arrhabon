//! RPC access to the program at confirmed commitment: account reads, blockhashes, sending.

use std::sync::Arc;

use anchor_client::compat::solana_rpc_client::nonblocking::rpc_client::RpcClient;
use anchor_client::compat::solana_transaction::Transaction;
use anchor_client::{
    Client, ClientError, Cluster, CommitmentConfig, Instruction, Program, RpcFilterType, Signer,
};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::clock::Clock as ClockSysvar;
use anchor_lang::solana_program::sysvar::SysvarId;
use anchor_lang::{AccountDeserialize, Discriminator};
use arrhabon::state::{self, id};
use arrhabon_app::PortError;
use arrhabon_app::ports::{PortResult, TxSignature, UnsignedTx};
use arrhabon_domain::{AccountId, Contribution, Vault};
use solana_keypair::Keypair;

use crate::error::{from_client, from_rpc};
use crate::{accounts, instructions};

pub struct Chain {
    program: Program<Arc<Keypair>>,
    rpc: RpcClient,
}

impl Chain {
    /// `payer` is the default signer anchor-client attaches; this adapter never relies on it.
    pub fn new(rpc_url: &str, payer: Arc<Keypair>) -> Result<Self, PortError> {
        let ws = rpc_url.replacen("http", "ws", 1);
        let cluster = Cluster::Custom(rpc_url.to_string(), ws);
        let client = Client::new_with_options(cluster, payer, CommitmentConfig::confirmed());
        let program = client.program(arrhabon::ID).map_err(from_client)?;
        let rpc =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        Ok(Chain { program, rpc })
    }

    pub async fn fetch<T: AccountDeserialize>(&self, key: Pubkey) -> PortResult<Option<T>> {
        match self.program.account::<T>(key).await {
            Ok(a) => Ok(Some(a)),
            Err(ClientError::AccountNotFound) => Ok(None),
            Err(e) => Err(from_client(e)),
        }
    }

    async fn all<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> PortResult<Vec<(Pubkey, T)>> {
        self.program
            .accounts::<T>(filters)
            .await
            .map_err(from_client)
    }

    pub async fn vaults(&self, filters: Vec<RpcFilterType>) -> PortResult<Vec<Vault>> {
        self.all::<state::Vault>(filters)
            .await?
            .iter()
            .map(|(k, v)| accounts::vault(k, v))
            .collect()
    }

    pub async fn contributions(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> PortResult<Vec<Contribution>> {
        self.all::<state::Contribution>(filters)
            .await?
            .iter()
            .map(|(_, c)| accounts::contribution(c))
            .collect()
    }

    /// anchor-client adds the account discriminator filter; `active_filter` drops inactive entries.
    pub async fn registered_universities(&self) -> PortResult<Vec<AccountId>> {
        let entries = self.all::<state::University>(vec![accounts::active_filter()]);
        Ok(entries.await?.iter().map(|(_, u)| id(&u.wallet)).collect())
    }

    pub async fn registered_attestors(&self) -> PortResult<Vec<AccountId>> {
        let entries = self.all::<state::Attestor>(vec![accounts::active_filter()]);
        Ok(entries.await?.iter().map(|(_, a)| id(&a.wallet)).collect())
    }

    pub async fn unsigned(&self, ixs: &[Instruction], payer: &Pubkey) -> PortResult<UnsignedTx> {
        let (hash, last_valid_block_height) = self
            .rpc
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            .await
            .map_err(from_rpc)?;
        Ok(UnsignedTx {
            bytes: instructions::unsigned_tx(ixs, payer, hash),
            last_valid_block_height,
        })
    }

    /// Returns once confirmed; a failed or expired transaction is an error.
    ///
    /// Blockhash expiry before confirmation maps to `Unavailable(EXPIRED_UNCONFIRMED)`
    /// (see `error::from_rpc`). Do not blindly resend: the status poll is at confirmed
    /// commitment, so the tx may have been processed and still confirm. Callers re-read
    /// the affected accounts first and retry only if the effect is absent.
    pub async fn send(&self, tx: &Transaction) -> PortResult<TxSignature> {
        self.rpc
            .send_and_confirm_transaction(tx)
            .await
            .map(|s| TxSignature(s.to_string()))
            .map_err(from_rpc)
    }

    /// Signs with `signer` (also the fee payer) and sends.
    pub async fn send_as(&self, ixs: &[Instruction], signer: &Keypair) -> PortResult<TxSignature> {
        let hash = self.rpc.get_latest_blockhash().await.map_err(from_rpc)?;
        let tx = Transaction::new_signed_with_payer(ixs, Some(&signer.pubkey()), &[signer], hash);
        self.send(&tx).await
    }

    /// Raw data of any account, `None` if it does not exist.
    pub async fn account_data(&self, key: &Pubkey) -> PortResult<Option<Vec<u8>>> {
        let res = self
            .rpc
            .get_account_with_commitment(key, CommitmentConfig::confirmed())
            .await
            .map_err(from_rpc)?;
        Ok(res.value.map(|a| a.data))
    }

    /// Lamports held by any address; 0 if it does not exist.
    pub async fn lamports(&self, key: &Pubkey) -> PortResult<u64> {
        let res = self
            .rpc
            .get_account_with_commitment(key, CommitmentConfig::confirmed())
            .await
            .map_err(from_rpc)?;
        Ok(res.value.map_or(0, |a| a.lamports))
    }

    /// An account with data; lamports sent to an unused address alone do not count.
    pub async fn has_data(&self, key: &Pubkey) -> PortResult<bool> {
        Ok(self.account_data(key).await?.is_some_and(|d| !d.is_empty()))
    }

    /// The Clock sysvar's unix timestamp: the time the program checks deadlines against.
    pub async fn chain_time(&self) -> PortResult<i64> {
        let account = self
            .rpc
            .get_account(&ClockSysvar::id())
            .await
            .map_err(from_rpc)?;
        clock_unix_timestamp(&account.data)
            .ok_or_else(|| PortError::rejected("malformed Clock sysvar"))
    }
}

/// Clock sysvar layout: slot, epoch_start_timestamp, epoch, leader_schedule_epoch, unix_timestamp.
pub fn clock_unix_timestamp(data: &[u8]) -> Option<i64> {
    let bytes = data.get(32..40)?;
    Some(i64::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::clock::Clock;

    #[test]
    fn reads_unix_timestamp_from_clock_sysvar_bytes() {
        let clock = Clock {
            slot: 1,
            epoch_start_timestamp: 2,
            epoch: 3,
            leader_schedule_epoch: 4,
            unix_timestamp: 1_761_400_000,
        };
        let data = bincode::serialize(&clock).unwrap();
        assert_eq!(clock_unix_timestamp(&data), Some(1_761_400_000));
        assert_eq!(clock_unix_timestamp(&data[..39]), None);
    }
}
