//! Adapter settings read from the environment; secrets never appear in `Debug`.

use std::fmt;
use std::time::Duration;

pub const AURORA_BASE_URL: &str = "https://intents-api.aurora.dev";
pub const ONECLICK_BASE_URL: &str = "https://1click.chaindefuser.com";

/// Which flavour of the same 1Click API to call.
#[derive(Clone, PartialEq, Eq)]
pub enum Api {
    /// Aurora Swap API: `/api/{op}/{api_key}`; the key is issued by Aurora Intents Studio.
    Aurora { api_key: String },
    /// NEAR Intents 1Click: `/v0/{op}`, optional partner JWT in `Authorization: Bearer`.
    OneClick { jwt: Option<String> },
}

impl fmt::Debug for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Api::Aurora { .. } => f.write_str("Aurora { api_key: <redacted> }"),
            Api::OneClick { jwt } => write!(
                f,
                "OneClick {{ jwt: {} }}",
                if jwt.is_some() { "<redacted>" } else { "none" }
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuroraConfig {
    pub api: Api,
    pub base_url: String,
    pub timeout: Duration,
    /// Off by default: quotes are `dry` (no deposit address reserved) unless this is set.
    /// Private so only `from_env` (or the `testing` feature) can turn it on.
    pub(crate) live: bool,
    /// How long a live quote's deposit address stays usable; sent as `deadline`.
    pub quote_ttl_secs: i64,
}

impl AuroraConfig {
    /// `AURORA_API_KEY` set → Aurora Swap API, else NEAR 1Click with optional `ONECLICK_JWT`.
    /// `AURORA_BASE_URL` overrides the default base; `AURORA_TIMEOUT_SECS` (default 15);
    /// `AURORA_LIVE=true` allows non-dry quotes.
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let api = match var("AURORA_API_KEY") {
            Some(api_key) => Api::Aurora { api_key },
            None => Api::OneClick {
                jwt: var("ONECLICK_JWT"),
            },
        };
        let default_base = match api {
            Api::Aurora { .. } => AURORA_BASE_URL,
            Api::OneClick { .. } => ONECLICK_BASE_URL,
        };
        Self {
            base_url: var("AURORA_BASE_URL").unwrap_or_else(|| default_base.to_owned()),
            timeout: parse_timeout(var("AURORA_TIMEOUT_SECS")),
            live: var("AURORA_LIVE").as_deref() == Some("true"),
            quote_ttl_secs: 30 * 60,
            api,
        }
    }

    /// Dry-only config against `base_url` with no credentials (tests, local mocks).
    pub fn dry(base_url: impl Into<String>) -> Self {
        Self {
            api: Api::OneClick { jwt: None },
            base_url: base_url.into(),
            timeout: Duration::from_secs(15),
            live: false,
            quote_ttl_secs: 30 * 60,
        }
    }

    /// Test-only escape hatch for exercising the live path against a mock.
    #[cfg(any(test, feature = "testing"))]
    #[doc(hidden)]
    pub fn with_live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
}

/// Whole seconds; missing, unparsable or zero falls back to 15 (zero would disable the timeout).
fn parse_timeout(raw: Option<String>) -> Duration {
    Duration::from_secs(
        raw.and_then(|s| s.parse().ok())
            .filter(|&n: &u64| n > 0)
            .unwrap_or(15),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_ignores_zero_and_garbage() {
        let t = |v: Option<&str>| parse_timeout(v.map(str::to_owned)).as_secs();
        assert_eq!(t(Some("30")), 30);
        assert_eq!(t(Some("0")), 15);
        assert_eq!(t(Some("abc")), 15);
        assert_eq!(t(None), 15);
    }
}
