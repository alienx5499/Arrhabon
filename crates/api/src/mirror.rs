//! DEMO ONLY: `POST /funding/mirror` mirrors a settled mainnet Aurora swap as devnet test USDC.

use std::collections::{HashMap, HashSet, VecDeque};

use arrhabon_app::address::to_base58;
use arrhabon_app::commands::{MirrorSettledSwap, SwapRecord};
use arrhabon_app::ports::TxSignature;
use arrhabon_domain::{AccountId, Usdc};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::error::{ApiError, Body};
use crate::routes::Res;
use crate::state::{AppState, Ports};

/// Bounded memory for remembered live quotes.
pub const MAX_REMEMBERED_QUOTES: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveQuoteRecord {
    pub funder: AccountId,
    pub vault: AccountId,
    pub min_usdc_out: Usdc,
}

/// Remembered quotes and once-only mirror claims.
#[derive(Debug, Default)]
pub struct MirrorState {
    quotes: HashMap<String, LiveQuoteRecord>,
    order: VecDeque<String>,
    mirrored: HashMap<String, TxSignature>,
    in_flight: HashSet<String>,
}

impl MirrorState {
    /// Remembers a live quote's details; drops the oldest if over capacity.
    pub fn remember(
        &mut self,
        deposit_address: String,
        funder: AccountId,
        vault: AccountId,
        min_usdc_out: Usdc,
    ) {
        if self.quotes.contains_key(&deposit_address) {
            return;
        }
        while self.order.len() >= MAX_REMEMBERED_QUOTES {
            if let Some(oldest) = self.order.pop_front() {
                self.quotes.remove(&oldest);
            }
        }
        self.order.push_back(deposit_address.clone());
        self.quotes.insert(
            deposit_address,
            LiveQuoteRecord {
                funder,
                vault,
                min_usdc_out,
            },
        );
    }

    /// Reserves a mirror attempt; refuses duplicate claims or unknown quotes.
    pub fn reserve(&mut self, deposit_address: &str) -> Result<LiveQuoteRecord, ApiError> {
        if let Some(sig) = self.mirrored.get(deposit_address) {
            return Err(ApiError::already_mirrored(&sig.0));
        }
        if self.in_flight.contains(deposit_address) {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "swap_pending",
                "mirror in progress; try again in a few seconds",
            ));
        }
        let record = self.quotes.get(deposit_address).cloned().ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "unknown_deposit",
                "unknown deposit address; only live quotes created by this API can be mirrored",
            )
        })?;
        self.in_flight.insert(deposit_address.to_owned());
        Ok(record)
    }

    /// Records successful completion with the transaction signature.
    pub fn complete(&mut self, deposit_address: &str, signature: TxSignature) {
        self.in_flight.remove(deposit_address);
        self.mirrored.insert(deposit_address.to_owned(), signature);
    }

    /// Releases a reservation if the use case failed.
    pub fn release(&mut self, deposit_address: &str) {
        self.in_flight.remove(deposit_address);
    }
}

#[derive(Deserialize)]
pub struct MirrorBody {
    pub deposit_address: String,
}

pub async fn mirror_swap<P: Ports>(State(s): State<AppState<P>>, Body(b): Body<MirrorBody>) -> Res {
    if !s.faucet_on || !s.aurora_live {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "mirror_disabled",
            "devnet mirror is disabled; requires FAUCET=true and AURORA_LIVE=true",
        ));
    }
    let faucet = s.faucet.get().ok_or_else(ApiError::chain_not_initialised)?;
    let record = s
        .mirror
        .lock()
        .expect("mirror lock poisoned")
        .reserve(&b.deposit_address)?;

    let cmd = MirrorSettledSwap {
        deposit_address: b.deposit_address.clone(),
        record: SwapRecord {
            funder: record.funder,
            vault: record.vault,
            amount: record.min_usdc_out,
        },
    };

    let mirror = match cmd.execute(&s.quoter, faucet).await {
        Ok(m) => m,
        Err(e) => {
            s.mirror
                .lock()
                .expect("mirror lock poisoned")
                .release(&b.deposit_address);
            return Err(e.into());
        }
    };

    s.mirror
        .lock()
        .expect("mirror lock poisoned")
        .complete(&b.deposit_address, mirror.signature.clone());

    tracing::info!(
        deposit_address = %b.deposit_address,
        funder = %to_base58(mirror.funder),
        vault = %to_base58(mirror.vault),
        usdc = mirror.usdc.micro(),
        "devnet mirror granted"
    );

    Ok(Json(json!({
        "demo": true,
        "notice": "Devnet mirror: the swap is real on mainnet; this is matching test USDC on devnet.",
        "signature": mirror.signature.0,
        "usdc_micro": mirror.usdc.micro().to_string(),
        "funder": to_base58(mirror.funder),
        "vault": to_base58(mirror.vault),
    })))
}
