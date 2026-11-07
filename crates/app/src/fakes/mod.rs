//! In-memory fakes of every port; the ledger applies domain transitions like the program does.

mod ledger;
mod state;

pub use ledger::FakeLedger;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use arrhabon_domain::{AccountId, Usdc};

use crate::PortError;
use crate::ports::{
    AttestorSigner, Clock, PortResult, QuoteRequest, SignedTx, SwapQuote, SwapQuoter, SwapStatus,
    TestFunds, TxSignature, UnsignedTx, WalletFunds,
};

/// A settable clock; clones share the same time.
#[derive(Debug, Clone, Default)]
pub struct FixedClock(Arc<AtomicI64>);

impl FixedClock {
    pub fn at(now: i64) -> Self {
        FixedClock(Arc::new(AtomicI64::new(now)))
    }
    pub fn set(&self, now: i64) {
        self.0.store(now, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Fake wire format: an unsigned tx is the pending action index; signing appends the signer.
pub fn sign_as(tx: UnsignedTx, signer: AccountId) -> SignedTx {
    let mut bytes = tx.bytes;
    bytes.extend_from_slice(&signer.0);
    SignedTx(bytes)
}

#[derive(Debug, Clone, Copy)]
pub struct FakeSigner(pub AccountId);

impl AttestorSigner for FakeSigner {
    fn attestor(&self) -> AccountId {
        self.0
    }

    async fn sign(&self, tx: UnsignedTx) -> PortResult<SignedTx> {
        Ok(sign_as(tx, self.0))
    }
}

/// Quotes a 6-decimal origin asset 1:1 into USDC minus a flat fee; quotes live 10 minutes.
#[derive(Debug)]
pub struct FakeQuoter {
    pub clock: FixedClock,
    pub fee: Usdc,
    statuses: Mutex<HashMap<String, SwapStatus>>,
}

impl FakeQuoter {
    pub fn new(clock: FixedClock, fee: Usdc) -> Self {
        FakeQuoter {
            clock,
            fee,
            statuses: Mutex::default(),
        }
    }

    pub fn set_status(&self, deposit_address: &str, status: SwapStatus) {
        self.statuses
            .lock()
            .expect("fake quoter lock poisoned")
            .insert(deposit_address.into(), status);
    }
}

impl SwapQuoter for FakeQuoter {
    async fn quote(&self, req: QuoteRequest) -> PortResult<SwapQuote> {
        let amount = u64::try_from(req.origin_amount)
            .map_err(|_| PortError::rejected("amount too large"))?;
        let min_usdc_out = Usdc::new(amount)
            .checked_sub(self.fee)
            .map_err(|_| PortError::rejected("amount below fee"))?;
        let mut statuses = self.statuses.lock().expect("fake quoter lock poisoned");
        let deposit_address = format!("deposit-{}", statuses.len() + 1);
        statuses.insert(deposit_address.clone(), SwapStatus::Pending);
        Ok(SwapQuote {
            deposit_address,
            min_usdc_out,
            fee: self.fee,
            expires_at: self.clock.now() + 600,
        })
    }

    async fn status(&self, deposit_address: &str) -> PortResult<SwapStatus> {
        let statuses = self.statuses.lock().expect("fake quoter lock poisoned");
        statuses
            .get(deposit_address)
            .copied()
            .ok_or_else(|| PortError::rejected("unknown deposit address"))
    }
}

/// A faucet over in-memory balances; `send` adds to the wallet and records the call.
#[derive(Debug, Clone, Default)]
pub struct FakeFaucet {
    balances: Arc<Mutex<HashMap<AccountId, WalletFunds>>>,
    sent: Arc<Mutex<Vec<(AccountId, u64, Usdc)>>>,
}

impl FakeFaucet {
    pub fn set(&self, wallet: AccountId, lamports: u64, usdc: Usdc) {
        let mut b = self.balances.lock().expect("fake faucet lock poisoned");
        b.insert(wallet, WalletFunds { lamports, usdc });
    }
    pub fn sent(&self) -> Vec<(AccountId, u64, Usdc)> {
        self.sent.lock().expect("fake faucet lock poisoned").clone()
    }
}

impl TestFunds for FakeFaucet {
    async fn funds(&self, wallet: AccountId) -> PortResult<WalletFunds> {
        let b = self.balances.lock().expect("fake faucet lock poisoned");
        Ok(b.get(&wallet).copied().unwrap_or(WalletFunds {
            lamports: 0,
            usdc: Usdc::new(0),
        }))
    }

    async fn send(&self, wallet: AccountId, lamports: u64, usdc: Usdc) -> PortResult<TxSignature> {
        let mut b = self.balances.lock().expect("fake faucet lock poisoned");
        let e = b.entry(wallet).or_insert(WalletFunds {
            lamports: 0,
            usdc: Usdc::new(0),
        });
        e.lamports += lamports;
        e.usdc = Usdc::new(e.usdc.micro() + usdc.micro());
        let mut sent = self.sent.lock().expect("fake faucet lock poisoned");
        sent.push((wallet, lamports, usdc));
        Ok(TxSignature(format!("faucet-{}", sent.len())))
    }
}
