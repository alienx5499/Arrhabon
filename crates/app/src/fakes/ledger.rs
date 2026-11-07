//! The program's state in memory, changed only through domain transitions.

use std::collections::BTreeSet;
use std::sync::{Mutex, MutexGuard};

use arrhabon_domain::{AccountId, Contribution, DomainError, Usdc, Vault, VaultStatus};

use super::FixedClock;
use super::state::{Action, State, rejected};
use crate::PortError;
use crate::ports::{
    BuiltVault, Clock, NewVault, PortResult, SignedTx, TxSignature, UnsignedTx, VaultLedger,
    VisaOutcome,
};

/// A built tx stays valid until this fake block height.
const LAST_VALID_BLOCK_HEIGHT: u64 = 1_150;

/// The program's state in memory; `fail_for` makes calls touching an account fail.
#[derive(Debug)]
pub struct FakeLedger {
    clock: FixedClock,
    state: Mutex<State>,
}

impl FakeLedger {
    pub fn new(clock: FixedClock) -> Self {
        FakeLedger {
            clock,
            state: Mutex::default(),
        }
    }

    pub fn put_vault(&self, vault: Vault) {
        self.lock().vaults.insert(vault.id(), vault);
    }

    pub fn put_contribution(&self, c: Contribution) {
        self.lock().contributions.insert((c.vault(), c.funder()), c);
    }

    /// Registered and active.
    pub fn register_university(&self, id: AccountId) {
        self.lock().universities.insert(id);
    }

    /// Registered and active.
    pub fn register_attestor(&self, id: AccountId) {
        self.lock().attestors.insert(id);
    }

    pub fn fail_for(&self, account: AccountId) {
        self.lock().failing.insert(account);
    }

    /// `owner` has no USDC token account (every other account has one).
    pub fn without_token_account(&self, owner: AccountId) {
        self.lock().without_token_account.insert(owner);
    }

    /// From now on, port reads return the current state while writes keep changing it.
    pub fn freeze_reads(&self) {
        let mut s = self.lock();
        s.frozen = Some((s.vaults.clone(), s.contributions.clone()));
    }

    /// Live state, ignoring `freeze_reads`.
    pub fn stored_vault(&self, id: AccountId) -> Option<Vault> {
        self.lock().vaults.get(&id).cloned()
    }

    /// Transactions built so far.
    pub fn queued(&self) -> usize {
        self.lock().pending.len()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("fake ledger lock poisoned")
    }

    fn vaults_where(&self, keep: impl Fn(&Vault) -> bool) -> Vec<Vault> {
        let s = self.lock();
        s.read().0.values().filter(|v| keep(v)).cloned().collect()
    }

    fn contributions_where(&self, keep: impl Fn(&Contribution) -> bool) -> Vec<Contribution> {
        let s = self.lock();
        s.read().1.values().filter(|c| keep(c)).cloned().collect()
    }

    fn queue(&self, action: Action) -> UnsignedTx {
        let mut s = self.lock();
        s.pending.push(Some(action));
        UnsignedTx {
            bytes: ((s.pending.len() - 1) as u64).to_le_bytes().to_vec(),
            last_valid_block_height: LAST_VALID_BLOCK_HEIGHT,
        }
    }
}

/// A domain error meaning the crank's goal is already reached becomes `AlreadyApplied`.
fn crank_error(e: DomainError, done: DomainError) -> PortError {
    match e == done {
        true => PortError::AlreadyApplied,
        false => rejected(e),
    }
}

impl VaultLedger for FakeLedger {
    fn vault_address(&self, student: AccountId, nonce: u64) -> AccountId {
        let mut id = [0xAA; 32];
        id[..8].copy_from_slice(&nonce.to_le_bytes());
        id[8..16].copy_from_slice(&student.0[..8]);
        AccountId(id)
    }

    async fn is_registered_university(&self, id: AccountId) -> PortResult<bool> {
        Ok(self.lock().universities.contains(&id))
    }

    async fn is_registered_attestor(&self, id: AccountId) -> PortResult<bool> {
        Ok(self.lock().attestors.contains(&id))
    }

    async fn registered_universities(&self) -> PortResult<Vec<AccountId>> {
        Ok(self.lock().universities.iter().copied().collect())
    }

    async fn registered_attestors(&self) -> PortResult<Vec<AccountId>> {
        Ok(self.lock().attestors.iter().copied().collect())
    }

    async fn vault(&self, id: AccountId) -> PortResult<Option<Vault>> {
        Ok(self.lock().read().0.get(&id).cloned())
    }

    async fn vaults_by_student(&self, student: AccountId) -> PortResult<Vec<Vault>> {
        Ok(self.vaults_where(|v| v.student() == student))
    }

    async fn vaults_by_university(&self, university: AccountId) -> PortResult<Vec<Vault>> {
        Ok(self.vaults_where(|v| v.university() == university))
    }

    async fn contributions(&self, vault: AccountId) -> PortResult<Vec<Contribution>> {
        Ok(self.contributions_where(|c| c.vault() == vault))
    }

    async fn contribution(
        &self,
        vault: AccountId,
        funder: AccountId,
    ) -> PortResult<Option<Contribution>> {
        Ok(self.lock().read().1.get(&(vault, funder)).cloned())
    }

    async fn contributions_by_funder(&self, funder: AccountId) -> PortResult<Vec<Contribution>> {
        Ok(self.contributions_where(|c| c.funder() == funder))
    }

    /// every Open vault, the superset the port allows, so ExpireDue's filter is exercised.
    async fn vaults_due_for_expiry(&self, _now: i64) -> PortResult<Vec<Vault>> {
        Ok(self.vaults_where(|v| v.status() == VaultStatus::Open))
    }

    async fn pending_refunds(&self) -> PortResult<Vec<Contribution>> {
        let refunding: BTreeSet<AccountId> = self
            .vaults_where(|v| v.status() == VaultStatus::Refunding)
            .iter()
            .map(Vault::id)
            .collect();
        Ok(self.contributions_where(|c| !c.refunded() && refunding.contains(&c.vault())))
    }

    async fn has_token_account(&self, owner: AccountId) -> PortResult<bool> {
        Ok(!self.lock().without_token_account.contains(&owner))
    }

    async fn build_create_vault(&self, vault: NewVault) -> PortResult<BuiltVault> {
        let id = self.vault_address(vault.student, vault.nonce);
        let tx = self.queue(Action::Create(vault, id));
        Ok(BuiltVault { vault: id, tx })
    }

    async fn build_contribute(
        &self,
        vault: AccountId,
        funder: AccountId,
        amount: Usdc,
    ) -> PortResult<UnsignedTx> {
        Ok(self.queue(Action::Contribute(vault, funder, amount)))
    }

    async fn build_attestation(
        &self,
        vault: AccountId,
        _attestor: AccountId,
        outcome: VisaOutcome,
    ) -> PortResult<UnsignedTx> {
        Ok(self.queue(Action::Attest(vault, outcome)))
    }

    /// Like a real chain, a transaction lands at most once, even if it fails.
    async fn submit(&self, tx: SignedTx) -> PortResult<TxSignature> {
        let malformed = || PortError::rejected("malformed transaction");
        let (index, signer) = tx.0.split_at_checked(8).ok_or_else(malformed)?;
        let index = u64::from_le_bytes(index.try_into().map_err(|_| malformed())?) as usize;
        let signer = AccountId(signer.try_into().map_err(|_| malformed())?);
        let now = self.clock.now();
        let mut s = self.lock();
        let slot = s.pending.get_mut(index).ok_or_else(malformed)?;
        let action = slot.take().ok_or(PortError::AlreadyApplied)?;
        s.apply(action, signer, now)?;
        Ok(s.signature())
    }

    async fn expire(&self, vault: AccountId) -> PortResult<TxSignature> {
        let now = self.clock.now();
        let mut s = self.lock();
        s.ensure_not_failing(&[vault])?;
        s.vault_mut(vault)?
            .expire(now)
            .map_err(|e| crank_error(e, DomainError::NotOpen))?;
        Ok(s.signature())
    }

    async fn refund(&self, vault: AccountId, funder: AccountId) -> PortResult<TxSignature> {
        let mut s = self.lock();
        s.ensure_not_failing(&[vault, funder])?;
        let mut c = s
            .contributions
            .get(&(vault, funder))
            .cloned()
            .ok_or_else(|| PortError::rejected("contribution account missing"))?;
        s.vault_mut(vault)?
            .refund(&mut c)
            .map_err(|e| crank_error(e, DomainError::AlreadyRefunded))?;
        s.contributions.insert((vault, funder), c);
        Ok(s.signature())
    }
}
