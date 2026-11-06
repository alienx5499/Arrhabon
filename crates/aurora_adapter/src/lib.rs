//! SwapQuoter adapter backed by Aurora's Swap API (NEAR Intents 1Click).
#![forbid(unsafe_code)]

mod config;
mod wire;

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use arrhabon_app::PortError;
use arrhabon_app::address::to_base58;
use arrhabon_app::ports::{
    DestinationAsset, PortResult, QuoteRequest, SwapQuote, SwapQuoter, SwapStatus,
};
use arrhabon_domain::Usdc;
use reqwest::{RequestBuilder, Response};
use serde::de::DeserializeOwned;

pub use config::{AURORA_BASE_URL, Api, AuroraConfig, ONECLICK_BASE_URL};
use wire::{QuoteBody, QuoteResponse, StatusResponse, Token, TokensResponse};

/// The mapped status plus the API's raw value, kept for logs and unknown states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapState {
    pub status: SwapStatus,
    pub raw: String,
}

pub struct AuroraQuoter {
    config: AuroraConfig,
    http: reqwest::Client,
    // cached for the process lifetime; add a TTL if Aurora lists tokens mid-run.
    tokens: Mutex<Option<Arc<Vec<Token>>>>,
}

impl AuroraQuoter {
    pub fn new(config: AuroraConfig) -> PortResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(transport)?;
        Ok(Self {
            config,
            http,
            tokens: Mutex::new(None),
        })
    }

    /// Indicative price only: no deposit address is reserved, so `deposit_address` is empty.
    pub async fn quote_dry(&self, req: QuoteRequest) -> PortResult<SwapQuote> {
        self.request_quote(req, true).await
    }

    /// Reserves a real deposit address; refused unless the config has `live` set.
    pub async fn quote_live(&self, req: QuoteRequest) -> PortResult<SwapQuote> {
        if !self.config.live {
            return Err(PortError::rejected(
                "live quotes are disabled; set AURORA_LIVE=true",
            ));
        }
        self.request_quote(req, false).await
    }

    pub async fn swap_state(&self, deposit_address: &str) -> PortResult<SwapState> {
        if deposit_address.is_empty() {
            return Err(PortError::rejected("dry quotes have no deposit address"));
        }
        let res: StatusResponse = self
            .send(
                self.get("status")
                    .query(&[("depositAddress", deposit_address)]),
            )
            .await?;
        Ok(SwapState {
            status: wire::map_status(&res.status),
            raw: res.status,
        })
    }

    async fn request_quote(&self, req: QuoteRequest, dry: bool) -> PortResult<SwapQuote> {
        if req.slippage_bps > 10_000 {
            return Err(PortError::rejected("slippage above 10,000 bps"));
        }
        let tokens = self.tokens().await?;
        let origin = wire::find_origin(&tokens, &req.origin_chain, &req.origin_asset)
            .map_err(PortError::rejected)?;
        let destination = match req.destination {
            DestinationAsset::SolanaUsdc => wire::find_solana_usdc(&tokens)
                .ok_or_else(|| PortError::rejected("USDC on Solana is not listed"))?,
        };
        let deadline = now() + self.config.quote_ttl_secs;
        // depositMode is left at the default SIMPLE, so the response's depositMemo is never needed.
        let body = QuoteBody {
            dry,
            swap_type: "EXACT_INPUT",
            slippage_tolerance: req.slippage_bps,
            origin_asset: origin.asset_id.clone(),
            deposit_type: "ORIGIN_CHAIN",
            destination_asset: destination.asset_id.clone(),
            amount: req.origin_amount.to_string(),
            refund_to: req.refund_to,
            refund_type: "ORIGIN_CHAIN",
            recipient: to_base58(req.recipient),
            recipient_type: "DESTINATION_CHAIN",
            deadline: wire::to_iso8601(deadline),
        };
        let res: QuoteResponse = self.send(self.post("quote").json(&body)).await?;
        to_swap_quote(res, dry, deadline)
    }

    /// Smallest-unit exponent of an origin token, matched like quote requests (asset id or symbol).
    pub async fn token_decimals(&self, chain: &str, asset: &str) -> PortResult<u8> {
        let tokens = self.tokens().await?;
        let token = wire::find_origin(&tokens, chain, asset).map_err(PortError::rejected)?;
        Ok(token.decimals)
    }

    async fn tokens(&self) -> PortResult<Arc<Vec<Token>>> {
        if let Some(t) = self.tokens.lock().expect("token cache poisoned").clone() {
            return Ok(t);
        }
        let res: TokensResponse = self.send(self.get("tokens")).await?;
        let tokens = Arc::new(res.into_tokens());
        *self.tokens.lock().expect("token cache poisoned") = Some(tokens.clone());
        Ok(tokens)
    }

    fn url(&self, op: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        match &self.config.api {
            Api::Aurora { api_key } => format!("{base}/api/{op}/{api_key}"),
            Api::OneClick { .. } => format!("{base}/v0/{op}"),
        }
    }

    fn authed(&self, rb: RequestBuilder) -> RequestBuilder {
        match &self.config.api {
            Api::OneClick { jwt: Some(jwt) } => rb.bearer_auth(jwt),
            _ => rb,
        }
    }

    fn secrets(&self) -> Vec<&str> {
        match &self.config.api {
            Api::Aurora { api_key } => vec![api_key],
            Api::OneClick { jwt } => jwt.iter().map(String::as_str).collect(),
        }
    }

    fn get(&self, op: &str) -> RequestBuilder {
        self.authed(self.http.get(self.url(op)))
    }

    fn post(&self, op: &str) -> RequestBuilder {
        self.authed(self.http.post(self.url(op)))
    }

    async fn send<T: DeserializeOwned>(&self, rb: RequestBuilder) -> PortResult<T> {
        let res = rb.send().await.map_err(transport)?;
        if !res.status().is_success() {
            return Err(http_error(res, &self.secrets()).await);
        }
        res.json().await.map_err(transport)
    }
}

impl SwapQuoter for AuroraQuoter {
    /// Dry unless the config is live.
    async fn quote(&self, req: QuoteRequest) -> PortResult<SwapQuote> {
        if self.config.live {
            self.quote_live(req).await
        } else {
            self.quote_dry(req).await
        }
    }

    async fn status(&self, deposit_address: &str) -> PortResult<SwapStatus> {
        Ok(self.swap_state(deposit_address).await?.status)
    }
}

fn to_swap_quote(res: QuoteResponse, dry: bool, deadline: i64) -> PortResult<SwapQuote> {
    let q = res.quote;
    let bad = |what: &str| PortError::rejected(format!("quote response: bad {what}"));
    let deposit_address = match (dry, q.deposit_address) {
        (true, _) => String::new(),
        (false, Some(a)) if !a.is_empty() => a,
        (false, _) => return Err(bad("depositAddress")),
    };
    let min_out: u64 = q.min_amount_out.parse().map_err(|_| bad("minAmountOut"))?;
    let usd = |v: &Option<String>| v.as_deref().and_then(wire::usd_to_micro);
    let fee = match (usd(&q.amount_in_usd), usd(&q.amount_out_usd)) {
        (Some(i), Some(o)) => i.saturating_sub(o),
        _ => 0,
    };
    let expires_at = q
        .time_when_inactive
        .as_deref()
        .and_then(wire::parse_iso8601)
        .map_or(deadline, |t| t.min(deadline));
    Ok(SwapQuote {
        deposit_address,
        min_usdc_out: Usdc::new(min_out),
        fee: Usdc::new(fee),
        expires_at,
    })
}

// `without_url`: the Aurora API key sits in the URL path and must never reach logs.
fn transport(e: reqwest::Error) -> PortError {
    let kind = if e.is_timeout() {
        "timeout"
    } else if e.is_decode() {
        return PortError::rejected(format!("bad response: {}", e.without_url()));
    } else {
        "transport"
    };
    PortError::Unavailable(format!("aurora {kind}: {}", e.without_url()))
}

/// Server text can echo the request path (and so the API key); scrub secrets, cap the length.
async fn http_error(res: Response, secrets: &[&str]) -> PortError {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    let message = serde_json::from_str::<wire::ErrorBody>(&text)
        .map(|b| b.message)
        .unwrap_or(text);
    let message = redact(&message, secrets);
    let message = format!("aurora {}: {message}", status.as_u16());
    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        PortError::Unavailable(message)
    } else {
        PortError::rejected(message)
    }
}

fn redact(text: &str, secrets: &[&str]) -> String {
    let scrubbed = secrets
        .iter()
        .filter(|s| !s.is_empty())
        .fold(text.to_owned(), |t, s| t.replace(s, "<redacted>"));
    scrubbed.chars().take(200).collect()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
