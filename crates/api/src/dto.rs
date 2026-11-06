//! JSON shapes of the app's views and inputs; amounts are USDC micro-units.

use arrhabon_app::address::parse_account;
use arrhabon_app::ports::{SwapQuote, SwapStatus, UnsignedTx};
use arrhabon_app::queries::{FunderView, RefundState, RegistryView, UniversityProof, VaultView};
use arrhabon_domain::{AccountId, VaultStatus};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};

use crate::error::ApiError;

pub fn account(text: &str) -> Result<AccountId, ApiError> {
    Ok(parse_account(text)?)
}

pub fn decode_tx(text: &str) -> Result<Vec<u8>, ApiError> {
    B64.decode(text.trim())
        .map_err(|_| ApiError::bad_request("tx must be standard base64"))
}

pub fn unsigned_tx(tx: &UnsignedTx) -> Value {
    json!({
        "unsigned_tx": B64.encode(&tx.bytes),
        "last_valid_block_height": tx.last_valid_block_height,
    })
}

fn status(s: VaultStatus) -> &'static str {
    match s {
        VaultStatus::Open => "open",
        VaultStatus::Released => "released",
        VaultStatus::Refunding => "refunding",
        VaultStatus::Closed => "closed",
    }
}

pub fn vault_view(v: &VaultView) -> Value {
    let contributors: Vec<Value> = v
        .contributors
        .iter()
        .map(|c| json!({ "funder": c.funder, "amount": c.amount, "refunded": c.refunded }))
        .collect();
    json!({
        "vault": v.vault,
        "student": v.student,
        "university": v.university,
        "attestor": v.attestor,
        "status": status(v.status),
        "target": v.target,
        "total_contributed": v.total_contributed,
        "deadline": v.deadline,
        "seconds_left": v.seconds_left,
        "contributors": contributors,
    })
}

pub fn funder_view(f: &FunderView) -> Value {
    let contributions: Vec<Value> = f
        .contributions
        .iter()
        .map(|c| {
            let refund = match c.refund {
                RefundState::NotDue => "not_due",
                RefundState::Pending => "pending",
                RefundState::Refunded => "refunded",
                RefundState::Unavailable => "unavailable",
            };
            json!({
                "vault": c.vault,
                "vault_status": c.vault_status.map(status),
                "amount": c.amount,
                "refund": refund,
            })
        })
        .collect();
    json!({ "funder": f.funder, "contributions": contributions })
}

pub fn university_proof(p: &UniversityProof) -> Value {
    json!({
        "vault": p.vault,
        "student": p.student,
        "status": status(p.status),
        "funds_held": p.funds_held,
        "target": p.target,
        "deadline": p.deadline,
        "attestor": p.attestor,
        "release_condition": p.release_condition,
    })
}

pub fn registry(r: &RegistryView) -> Value {
    let entries =
        |ids: &[String]| -> Vec<Value> { ids.iter().map(|a| json!({ "address": a })).collect() };
    json!({ "universities": entries(&r.universities), "attestors": entries(&r.attestors) })
}

pub fn swap_quote(q: &SwapQuote) -> Value {
    json!({
        // A dry quote reserves no deposit address: price only, nothing to send money to.
        "dry": q.deposit_address.is_empty(),
        "deposit_address": q.deposit_address,
        // Guaranteed floor (Aurora's minAmountOut): at least this arrives, or the swap refunds.
        "min_usdc_out": q.min_usdc_out.micro(),
        "fee": q.fee.micro(),
        "expires_at": q.expires_at,
    })
}

pub fn swap_status(s: SwapStatus) -> &'static str {
    match s {
        SwapStatus::Pending => "pending",
        SwapStatus::Success => "success",
        SwapStatus::Refunded => "refunded",
        SwapStatus::Failed => "failed",
    }
}
