//! Cluster CLI (devnet, or localnet via `ARRHABON_CLUSTER`): `setup` initialises Config and the
//! registry, `run` drives both demo paths, `sign <party>` signs an API-built transaction,
//! `registry <attestor|university> <pubkey> --active=<bool>` switches a registry entry,
//! `token-metadata --uri <url>` names the test mint.
#![forbid(unsafe_code)]

mod registry;
mod run;
mod setup;
mod token_metadata;

use std::error::Error;
use std::io::{Read, Write};
use std::process::ExitCode;

use anchor_client::Signer;
use arrhabon_app::ports::UnsignedTx;
use arrhabon_solana_adapter::{Keypair, Network, ensure_cluster, read_keypair_file, sign_tx};

type Res<T = ()> = Result<T, Box<dyn Error>>;

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Project keypairs from `.keys/` (written by `scripts/devnet_setup.sh`); never printed.
pub struct Keys {
    pub deployer: Keypair,
    pub crank: Keypair,
    pub attestor: Keypair,
    pub university: Keypair,
    pub student: Keypair,
    pub funders: [Keypair; 2],
    pub mint: Keypair,
}

fn key(name: &str) -> Res<Keypair> {
    let path = format!("{REPO}/.keys/{name}.json");
    read_keypair_file(&path).map_err(|e| format!("reading {path}: {e}").into())
}

impl Keys {
    fn load() -> Res<Self> {
        Ok(Keys {
            deployer: key("deployer")?,
            crank: key("crank")?,
            attestor: key("attestor")?,
            university: key("university")?,
            student: key("student")?,
            funders: [key("funder1")?, key("funder2")?],
            mint: key("usdc-mint")?,
        })
    }
}

/// The cluster and its RPC (`SOLANA_RPC_URL` or the cluster default), checked by the guard.
async fn cluster() -> Res<(Network, String)> {
    let network = Network::from_env()?;
    let url = network.rpc_url();
    ensure_cluster(network, &url)
        .await
        .map_err(|e| e.to_string())?;
    Ok((network, url))
}

/// Raw unsigned transaction bytes on stdin, the party's signed bytes on stdout; no RPC.
fn sign(party: &str) -> Res {
    if !["student", "funder1", "funder2"].contains(&party) {
        return Err("sign: party must be student, funder1 or funder2".into());
    }
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes)?;
    let unsigned = UnsignedTx {
        bytes,
        last_valid_block_height: 0,
    };
    std::io::stdout().write_all(&sign_tx(&unsigned, &key(party)?)?.0)?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let result = async {
        if cmd == "sign" {
            return sign(&std::env::args().nth(2).unwrap_or_default());
        }
        if cmd == "registry" {
            let switch = registry::parse(&std::env::args().skip(2).collect::<Vec<_>>())?;
            let admin = key("deployer")?;
            let (_, url) = cluster().await?;
            return registry::run(&switch, &admin, &url).await;
        }
        if cmd == "token-metadata" {
            let uri = token_metadata::parse(&std::env::args().skip(2).collect::<Vec<_>>())?;
            let (deployer, mint) = (key("deployer")?, key("usdc-mint")?.pubkey());
            let (net, url) = cluster().await?;
            return token_metadata::run(uri, &deployer, mint, net, &url).await;
        }
        let (keys, (net, url)) = (Keys::load()?, cluster().await?);
        match cmd.as_str() {
            "setup" => setup::run(&keys, net, &url).await,
            "run" => run::run(keys, net, &url).await,
            _ => Err(
                "usage: [ARRHABON_CLUSTER=localnet] devnet <setup|run|sign PARTY|registry ...|token-metadata --uri URL>"
                    .into(),
            ),
        }
    }
    .await;
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
