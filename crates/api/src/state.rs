//! Shared handler state: the port implementations chosen at startup.

use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};

use arrhabon_app::ports::{AttestorSigner, Clock, SwapQuoter, TestFunds, VaultLedger};

use crate::error::ApiError;

/// The concrete port types one running server uses (live adapters, or fakes in tests).
pub trait Ports: Send + Sync + 'static {
    type Ledger: VaultLedger + 'static;
    type Quoter: SwapQuoter + 'static;
    type Clock: Clock + 'static;
    type Attestor: AttestorSigner + 'static;
    type Faucet: TestFunds + 'static;

    /// Called before each crank tick; the live clock re-reads chain time here.
    fn refresh_clock(
        _ledger: &Self::Ledger,
        _clock: &Self::Clock,
    ) -> impl Future<Output = ()> + Send {
        async {}
    }
}

pub struct Inner<P: Ports> {
    /// Empty until the program's Config is readable on chain; set once by startup.
    pub ledger: OnceLock<P::Ledger>,
    pub quoter: P::Quoter,
    pub clock: P::Clock,
    /// The DEMO attestor; `None` unless `DEMO_ATTESTOR=true`.
    pub attestor: Option<P::Attestor>,
    pub crank: bool,
    /// DEMO faucet, `FAUCET=true` only; set once the chain is ready, like the ledger.
    pub faucet_on: bool,
    pub faucet: OnceLock<P::Faucet>,
    /// Faucet cooldowns and the hourly grant budget.
    pub faucet_claims: Mutex<crate::faucet::Claims>,
    /// `devnet` or `localnet`, reported by `/health`.
    pub cluster: &'static str,
}

pub struct AppState<P: Ports>(pub Arc<Inner<P>>);

impl<P: Ports> Clone for AppState<P> {
    fn clone(&self) -> Self {
        AppState(self.0.clone())
    }
}

impl<P: Ports> std::ops::Deref for AppState<P> {
    type Target = Inner<P>;
    fn deref(&self) -> &Inner<P> {
        &self.0
    }
}

impl<P: Ports> Inner<P> {
    pub fn ledger(&self) -> Result<&P::Ledger, ApiError> {
        self.ledger
            .get()
            .ok_or_else(ApiError::chain_not_initialised)
    }
}
