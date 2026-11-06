//! 1Click JSON shapes and the pure mappings between them and the app's port types.

use arrhabon_app::ports::SwapStatus;
use serde::{Deserialize, Serialize};

/// USDC on Solana mainnet, the only destination vaults accept.
pub const SOLANA_USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOLANA_CHAIN: &str = "sol";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub asset_id: String,
    pub blockchain: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(default)]
    pub contract_address: Option<String>,
}

/// Aurora wraps the list as `{ tokens: [...] }`; raw 1Click returns the bare array.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum TokensResponse {
    Wrapped { tokens: Vec<Token> },
    Bare(Vec<Token>),
}

impl TokensResponse {
    pub fn into_tokens(self) -> Vec<Token> {
        match self {
            TokensResponse::Wrapped { tokens } | TokensResponse::Bare(tokens) => tokens,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteBody {
    pub dry: bool,
    pub swap_type: &'static str,
    pub slippage_tolerance: u16,
    pub origin_asset: String,
    pub deposit_type: &'static str,
    pub destination_asset: String,
    pub amount: String,
    pub refund_to: String,
    pub refund_type: &'static str,
    pub recipient: String,
    pub recipient_type: &'static str,
    pub deadline: String,
}

#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    pub quote: Quote,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Post-slippage floor the API guarantees; the expected (not guaranteed) `amountOut` is unused.
    pub min_amount_out: String,
    #[serde(default)]
    pub amount_in_usd: Option<String>,
    #[serde(default)]
    pub amount_out_usd: Option<String>,
    /// Absent on dry quotes.
    #[serde(default)]
    pub deposit_address: Option<String>,
    #[serde(default)]
    pub time_when_inactive: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    pub message: String,
}

/// Every in-flight state, and anything new the API adds, is `Pending`.
pub fn map_status(raw: &str) -> SwapStatus {
    match raw {
        "SUCCESS" => SwapStatus::Success,
        "REFUNDED" => SwapStatus::Refunded,
        "FAILED" => SwapStatus::Failed,
        // PENDING_DEPOSIT, KNOWN_DEPOSIT_TX, PROCESSING, INCOMPLETE_DEPOSIT (refunded after
        // the deadline unless topped up), and unknown values.
        _ => SwapStatus::Pending,
    }
}

/// Origin token by chain plus exact asset id or a unique symbol.
pub fn find_origin<'a>(tokens: &'a [Token], chain: &str, asset: &str) -> Result<&'a Token, String> {
    let on_chain = || tokens.iter().filter(|t| t.blockchain == chain);
    if let Some(t) = on_chain().find(|t| t.asset_id == asset) {
        return Ok(t);
    }
    let mut by_symbol = on_chain().filter(|t| t.symbol.eq_ignore_ascii_case(asset));
    match (by_symbol.next(), by_symbol.next()) {
        (Some(t), None) => Ok(t),
        (Some(_), Some(_)) => Err(format!(
            "{asset} on {chain} is ambiguous; pass the asset id"
        )),
        (None, _) => Err(format!("{asset} on {chain} is not supported")),
    }
}

pub fn find_solana_usdc(tokens: &[Token]) -> Option<&Token> {
    tokens.iter().find(|t| {
        t.blockchain == SOLANA_CHAIN && t.contract_address.as_deref() == Some(SOLANA_USDC_MINT)
    })
}

/// A decimal USD string ("4.99755") in micro-units, truncated; no floats.
pub fn usd_to_micro(s: &str) -> Option<u64> {
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    let frac: String = frac.chars().chain(std::iter::repeat('0')).take(6).collect();
    let int: u64 = if int.is_empty() { 0 } else { int.parse().ok()? };
    int.checked_mul(1_000_000)?.checked_add(frac.parse().ok()?)
}

// Howard Hinnant's civil-date algorithms, proleptic Gregorian, UTC.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    era * 146_097 + (yoe * 365 + yoe / 4 - yoe / 100 + doy) - 719_468
}

/// Unix seconds as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn to_iso8601(unix: i64) -> String {
    let (days, secs) = (unix.div_euclid(86_400), unix.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    let (h, mi, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Parses `YYYY-MM-DDTHH:MM:SS[.fff]Z` into unix seconds (fraction dropped).
pub fn parse_iso8601(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || !s.ends_with('Z') || b[4] != b'-' || b[10] != b'T' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let days = days_from_civil(n(0..4)?, n(5..7)?, n(8..10)?);
    Some(days * 86_400 + n(11..13)? * 3600 + n(14..16)? * 60 + n(17..19)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_map_to_the_app_enum() {
        assert_eq!(map_status("SUCCESS"), SwapStatus::Success);
        assert_eq!(map_status("REFUNDED"), SwapStatus::Refunded);
        assert_eq!(map_status("FAILED"), SwapStatus::Failed);
        for raw in [
            "PENDING_DEPOSIT",
            "KNOWN_DEPOSIT_TX",
            "PROCESSING",
            "INCOMPLETE_DEPOSIT",
            "NEW_THING",
        ] {
            assert_eq!(map_status(raw), SwapStatus::Pending, "{raw}");
        }
    }

    #[test]
    fn usd_strings_become_micro_units() {
        assert_eq!(usd_to_micro("4.997550000000"), Some(4_997_550));
        assert_eq!(usd_to_micro("5"), Some(5_000_000));
        assert_eq!(usd_to_micro(".5"), Some(500_000));
        assert_eq!(usd_to_micro("abc"), None);
        assert_eq!(usd_to_micro("-1"), None);
    }

    #[test]
    fn iso8601_round_trips() {
        assert_eq!(to_iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(to_iso8601(1_793_000_000), "2026-10-26T07:33:20Z");
        assert_eq!(
            parse_iso8601("2026-10-26T07:33:20.000Z"),
            Some(1_793_000_000)
        );
        assert_eq!(parse_iso8601("2024-02-29T23:59:59Z"), Some(1_709_251_199));
        assert_eq!(parse_iso8601("not a date"), None);
    }
}
