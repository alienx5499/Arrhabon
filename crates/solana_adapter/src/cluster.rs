//! Which cluster the adapter may use: devnet by genesis hash, localnet on loopback only, never mainnet.

use anchor_client::compat::solana_rpc_client::nonblocking::rpc_client::RpcClient;
use arrhabon_app::PortError;
use arrhabon_app::ports::PortResult;

use crate::error::from_rpc;

/// Public devnet RPC, used when `SOLANA_RPC_URL` is unset on devnet.
pub const DEVNET_RPC: &str = "https://api.devnet.solana.com";
/// `solana-test-validator`'s default RPC, used when `SOLANA_RPC_URL` is unset on localnet.
pub const LOCALNET_RPC: &str = "http://127.0.0.1:8899";
pub const DEVNET_GENESIS: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
pub const MAINNET_GENESIS: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

/// The cluster chosen with `ARRHABON_CLUSTER` (`devnet` by default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    Devnet,
    /// A `solana-test-validator` on this machine; its genesis is fresh per ledger.
    Localnet,
}

impl Network {
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.map(str::trim) {
            None | Some("" | "devnet") => Ok(Network::Devnet),
            Some("localnet") => Ok(Network::Localnet),
            Some(other) => Err(format!(
                "ARRHABON_CLUSTER must be devnet or localnet, got {other:?}"
            )),
        }
    }

    pub fn from_env() -> Result<Self, String> {
        Self::parse(std::env::var("ARRHABON_CLUSTER").ok().as_deref())
    }

    pub fn name(self) -> &'static str {
        match self {
            Network::Devnet => "devnet",
            Network::Localnet => "localnet",
        }
    }

    /// `SOLANA_RPC_URL` if set, else this cluster's default endpoint.
    pub fn rpc_url(self) -> String {
        std::env::var("SOLANA_RPC_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| self.default_rpc().to_owned())
    }

    pub fn default_rpc(self) -> &'static str {
        match self {
            Network::Devnet => DEVNET_RPC,
            Network::Localnet => LOCALNET_RPC,
        }
    }

    /// Explorer link; on localnet it points the explorer at the default local RPC.
    pub fn explorer_tx(self, signature: &str) -> String {
        match self {
            Network::Devnet => format!("https://explorer.solana.com/tx/{signature}?cluster=devnet"),
            Network::Localnet => format!(
                "https://explorer.solana.com/tx/{signature}\
                 ?cluster=custom&customUrl=http%3A%2F%2F127.0.0.1%3A8899"
            ),
        }
    }

    /// Before any request: localnet only on a loopback host. The URL is never echoed
    /// (it may carry an API key).
    pub fn check_url(self, rpc_url: &str) -> Result<(), String> {
        match (self, is_loopback(rpc_url)) {
            (Network::Devnet, _) | (Network::Localnet, true) => Ok(()),
            (Network::Localnet, false) => Err("ARRHABON_CLUSTER=localnet needs SOLANA_RPC_URL \
                 on host 127.0.0.1 or localhost; refusing a remote RPC"
                .into()),
        }
    }

    /// After reading the genesis hash: mainnet always refused, devnet must match.
    pub fn check_genesis(self, genesis: &str) -> Result<(), String> {
        match (self, genesis) {
            (_, MAINNET_GENESIS) => Err("SOLANA_RPC_URL is mainnet; refusing".into()),
            (Network::Devnet, DEVNET_GENESIS) | (Network::Localnet, _) => Ok(()),
            (Network::Devnet, other) => Err(format!(
                "SOLANA_RPC_URL is not devnet: genesis {other}, expected {DEVNET_GENESIS}"
            )),
        }
    }
}

/// True when the URL's host is exactly `127.0.0.1` or `localhost` (userinfo tricks rejected).
fn is_loopback(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return false;
    }
    let host = authority.split(':').next().unwrap_or_default();
    host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost")
}

/// `Ok` only for an RPC the chosen cluster allows: `Rejected` for any other cluster,
/// `Unavailable` when the RPC cannot be reached.
pub async fn ensure_cluster(network: Network, rpc_url: &str) -> PortResult<()> {
    network.check_url(rpc_url).map_err(PortError::rejected)?;
    let genesis = RpcClient::new(rpc_url.to_string())
        .get_genesis_hash()
        .await
        .map_err(from_rpc)?
        .to_string();
    network.check_genesis(&genesis).map_err(PortError::rejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_GENESIS: &str = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY";

    #[test]
    fn parses_cluster_with_devnet_default() {
        assert_eq!(Network::parse(None), Ok(Network::Devnet));
        assert_eq!(Network::parse(Some("localnet")), Ok(Network::Localnet));
        assert!(Network::parse(Some("mainnet-beta")).is_err());
    }

    #[test]
    fn localnet_accepts_only_loopback_hosts() {
        for ok in [
            "http://127.0.0.1:8899",
            "http://localhost:8899/",
            "http://LOCALHOST",
        ] {
            assert_eq!(Network::Localnet.check_url(ok), Ok(()), "{ok}");
        }
        for bad in [
            "https://api.devnet.solana.com",
            "https://api.mainnet-beta.solana.com",
            "http://127.0.0.1.evil.com:8899",
            "http://localhost@evil.com",
            "http://evil.com/127.0.0.1",
            "127.0.0.1:8899",
        ] {
            let e = Network::Localnet.check_url(bad).unwrap_err();
            assert!(
                e.contains("127.0.0.1 or localhost") && !e.contains(bad),
                "{bad}"
            );
        }
        assert_eq!(Network::Devnet.check_url("https://rpc.example"), Ok(()));
    }

    #[test]
    fn mainnet_genesis_is_always_refused() {
        for net in [Network::Devnet, Network::Localnet] {
            assert!(
                net.check_genesis(MAINNET_GENESIS)
                    .unwrap_err()
                    .contains("mainnet")
            );
        }
    }

    #[test]
    fn devnet_keeps_its_genesis_check() {
        assert_eq!(Network::Devnet.check_genesis(DEVNET_GENESIS), Ok(()));
        assert!(Network::Devnet.check_genesis(LOCAL_GENESIS).is_err());
        assert_eq!(Network::Localnet.check_genesis(LOCAL_GENESIS), Ok(()));
    }
}
