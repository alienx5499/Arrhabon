//! Startup settings from the environment; each parser is pure so bad values fail with a message.

use std::time::Duration;

use axum::http::{HeaderValue, Uri};

/// Floor for `CRANK_INTERVAL_SECS`: a tighter loop only burns RPC quota.
pub const MIN_CRANK_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_CRANK_INTERVAL: Duration = Duration::from_secs(10);

/// A set, non-blank environment variable.
pub fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

pub fn flag(name: &str, value: Option<&str>, default: bool) -> Result<bool, String> {
    match value {
        None => Ok(default),
        Some("true" | "1") => Ok(true),
        Some("false" | "0") => Ok(false),
        Some(_) => Err(format!("{name} must be true or false")),
    }
}

/// Seconds between crank runs; never below `MIN_CRANK_INTERVAL`.
pub fn crank_interval(value: Option<&str>) -> Result<Duration, String> {
    let every = match value {
        None => DEFAULT_CRANK_INTERVAL,
        Some(s) => Duration::from_secs(
            s.trim()
                .parse()
                .map_err(|_| "CRANK_INTERVAL_SECS must be whole seconds")?,
        ),
    };
    Ok(every.max(MIN_CRANK_INTERVAL))
}

/// A browser origin: `http(s)://host[:port]`, nothing after it, no wildcard.
pub fn web_origin(value: &str) -> Result<HeaderValue, String> {
    let bad =
        || format!("WEB_ORIGIN must be an origin like https://app.example.com, got {value:?}");
    let uri: Uri = value.parse().map_err(|_| bad())?;
    let (Some(scheme), Some(authority)) = (uri.scheme_str(), uri.authority()) else {
        return Err(bad());
    };
    let exact = format!("{scheme}://{authority}") == value;
    match (scheme, exact && !authority.as_str().contains(['@', '*'])) {
        ("http" | "https", true) => HeaderValue::from_str(value).map_err(|_| bad()),
        _ => Err(bad()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_accept_only_boolean_spellings() {
        assert_eq!(flag("CRANK", None, true), Ok(true));
        assert_eq!(flag("CRANK", Some("0"), true), Ok(false));
        assert_eq!(flag("CRANK", Some("true"), false), Ok(true));
        assert!(
            flag("CRANK", Some("yes"), false)
                .unwrap_err()
                .contains("CRANK")
        );
    }

    #[test]
    fn crank_interval_has_a_floor_and_rejects_junk() {
        assert_eq!(crank_interval(None), Ok(Duration::from_secs(10)));
        assert_eq!(crank_interval(Some("1")), Ok(MIN_CRANK_INTERVAL));
        assert_eq!(crank_interval(Some("30")), Ok(Duration::from_secs(30)));
        assert!(crank_interval(Some("-3")).is_err());
        assert!(crank_interval(Some("5s")).is_err());
    }

    #[test]
    fn web_origin_must_be_a_plain_http_origin() {
        for ok in ["http://localhost:3000", "https://arrhabon.example.com"] {
            assert_eq!(web_origin(ok).unwrap(), ok);
        }
        for bad in [
            "*",
            "localhost:3000",
            "https://app.example.com/",
            "https://app.example.com/path",
            "ftp://app.example.com",
            "https://user@app.example.com",
            "https://*.example.com",
            "",
        ] {
            assert!(web_origin(bad).is_err(), "{bad}");
        }
    }
}
