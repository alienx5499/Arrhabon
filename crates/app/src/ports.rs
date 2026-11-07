//! Ports the use cases depend on; adapters (Solana, Aurora, API) implement them.

use std::future::Future;

use arrhabon_domain::{AccountId, Contribution, Deadline, Usdc, Vault};

use crate::PortError;

pub type PortResult<T> = Result<T, PortError>;

/// A serialized transaction still missing the signature of its required signer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsignedTx {
    pub bytes: Vec<u8>,
    /// The chain drops the tx once past this block height; the wallet must rebuild it then.
    pub last_valid_block_height: u64,
}

/// A serialized transaction ready to submit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedTx(pub Vec<u8>);

/// The chain's id for a submitted transaction (base58 on Solana).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxSignature(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewVault {
    pub student: AccountId,
    pub university: AccountId,
    pub attestor: AccountId,
    pub target: Usdc,
    pub deadline: Deadline,
    /// Lets one student hold several vaults; part of the vault address.
    pub nonce: u64,
}

/// A built `create_vault` transaction and the address of the vault it creates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltVault {
    pub vault: AccountId,
    pub tx: UnsignedTx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisaOutcome {
    Issued,
    Denied,
}

/// Program state on chain: reads, transaction building, and permissionless crank calls.
///
/// Contract for `submit`, `expire` and `refund`: they return only once the transaction
/// reached confirmed commitment; a dropped or failed transaction is an error, never `Ok`.
/// When the chain already shows the effect (the same transaction processed, the vault no
/// longer Open, the contribution already refunded) they return `PortError::AlreadyApplied`,
/// so crank reruns on stale reads stay idempotent. Program errors come back as
/// `PortError::Rejected` with the matching `DomainError` code.
pub trait VaultLedger: Send + Sync {
    /// The address `build_create_vault` will use (the PDA); derived, no I/O.
    fn vault_address(&self, student: AccountId, nonce: u64) -> AccountId;
    /// Registered and active in the program's registry.
    fn is_registered_university(
        &self,
        id: AccountId,
    ) -> impl Future<Output = PortResult<bool>> + Send;
    /// Registered and active in the program's registry.
    fn is_registered_attestor(
        &self,
        id: AccountId,
    ) -> impl Future<Output = PortResult<bool>> + Send;
    /// Wallets of active registry entries; the program stores no names.
    fn registered_universities(&self) -> impl Future<Output = PortResult<Vec<AccountId>>> + Send;
    /// Wallets of active registry entries; the program stores no names.
    fn registered_attestors(&self) -> impl Future<Output = PortResult<Vec<AccountId>>> + Send;
    fn vault(&self, id: AccountId) -> impl Future<Output = PortResult<Option<Vault>>> + Send;
    fn vaults_by_student(
        &self,
        student: AccountId,
    ) -> impl Future<Output = PortResult<Vec<Vault>>> + Send;
    fn vaults_by_university(
        &self,
        university: AccountId,
    ) -> impl Future<Output = PortResult<Vec<Vault>>> + Send;
    fn contributions(
        &self,
        vault: AccountId,
    ) -> impl Future<Output = PortResult<Vec<Contribution>>> + Send;
    fn contribution(
        &self,
        vault: AccountId,
        funder: AccountId,
    ) -> impl Future<Output = PortResult<Option<Contribution>>> + Send;
    fn contributions_by_funder(
        &self,
        funder: AccountId,
    ) -> impl Future<Output = PortResult<Vec<Contribution>>> + Send;
    /// Open vaults that may be past their deadline; a superset is fine, `ExpireDue` filters.
    fn vaults_due_for_expiry(
        &self,
        now: i64,
    ) -> impl Future<Output = PortResult<Vec<Vault>>> + Send;
    /// Unrefunded contributions of Refunding vaults.
    fn pending_refunds(&self) -> impl Future<Output = PortResult<Vec<Contribution>>> + Send;
    /// Whether `owner` already has a USDC token account at its associated address.
    fn has_token_account(&self, owner: AccountId) -> impl Future<Output = PortResult<bool>> + Send;

    /// Signed by the student.
    fn build_create_vault(
        &self,
        vault: NewVault,
    ) -> impl Future<Output = PortResult<BuiltVault>> + Send;
    /// Signed by the funder.
    fn build_contribute(
        &self,
        vault: AccountId,
        funder: AccountId,
        amount: Usdc,
    ) -> impl Future<Output = PortResult<UnsignedTx>> + Send;
    /// `confirm_visa` or `deny_visa`, signed by the attestor.
    fn build_attestation(
        &self,
        vault: AccountId,
        attestor: AccountId,
        outcome: VisaOutcome,
    ) -> impl Future<Output = PortResult<UnsignedTx>> + Send;
    fn submit(&self, tx: SignedTx) -> impl Future<Output = PortResult<TxSignature>> + Send;
    /// Permissionless; the adapter pays the fee.
    fn expire(&self, vault: AccountId) -> impl Future<Output = PortResult<TxSignature>> + Send;
    /// Permissionless; the adapter pays the fee.
    fn refund(
        &self,
        vault: AccountId,
        funder: AccountId,
    ) -> impl Future<Output = PortResult<TxSignature>> + Send;
}

/// The demo attestor's key: signs `confirm_visa` / `deny_visa` transactions.
pub trait AttestorSigner: Send + Sync {
    fn attestor(&self) -> AccountId;
    fn sign(&self, tx: UnsignedTx) -> impl Future<Output = PortResult<SignedTx>> + Send;
}

/// What a cross-chain swap delivers; fixed, the vaults only take USDC on Solana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationAsset {
    SolanaUsdc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteRequest {
    pub origin_chain: String,
    pub origin_asset: String,
    /// In the origin asset's smallest unit.
    pub origin_amount: u128,
    /// Solana wallet that receives the USDC and then contributes.
    pub recipient: AccountId,
    /// Origin-chain address that gets the money back if the swap fails.
    pub refund_to: String,
    /// Basis points; at most 10,000.
    pub slippage_bps: u16,
    pub destination: DestinationAsset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapQuote {
    pub deposit_address: String,
    /// Guaranteed floor: the swap delivers at least this much or refunds.
    pub min_usdc_out: Usdc,
    pub fee: Usdc,
    /// Unix seconds after which the deposit address must not be used.
    pub expires_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapStatus {
    Pending,
    Success,
    Refunded,
    Failed,
}

/// Cross-chain funding into USDC on Solana.
pub trait SwapQuoter: Send + Sync {
    fn quote(&self, req: QuoteRequest) -> impl Future<Output = PortResult<SwapQuote>> + Send;
    fn status(&self, deposit_address: &str) -> impl Future<Output = PortResult<SwapStatus>> + Send;
}

/// What a wallet holds of the two things a demo user needs: SOL for fees and test USDC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletFunds {
    pub lamports: u64,
    pub usdc: Usdc,
}

/// DEMO ONLY: a devnet faucet wallet that tops up judges' wallets with SOL and test USDC.
pub trait TestFunds: Send + Sync {
    fn funds(&self, wallet: AccountId) -> impl Future<Output = PortResult<WalletFunds>> + Send;
    /// One transaction from the faucet wallet; either amount may be zero, not both.
    fn send(
        &self,
        wallet: AccountId,
        lamports: u64,
        usdc: Usdc,
    ) -> impl Future<Output = PortResult<TxSignature>> + Send;
}

/// Unix seconds.
pub trait Clock: Send + Sync {
    fn now(&self) -> i64;
}
