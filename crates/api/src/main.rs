//! HTTP composition root: wires adapters into use cases and serves them.
#![forbid(unsafe_code)]

mod config;
mod crank;
mod dto;
mod error;
mod faucet;
mod routes;
mod state;
#[cfg(test)]
mod tests;

use std::process::ExitCode;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use arrhabon_app::PortError;
use arrhabon_aurora_adapter::{AuroraConfig, AuroraQuoter};
use arrhabon_solana_adapter::{
    Chain, ChainClock, Keypair, KeypairAttestor, Network, SolanaFaucet, SolanaLedger,
    ensure_cluster, read_keypair_file,
};
use tokio::sync::watch;
use tracing_subscriber::EnvFilter;

use crate::config::{crank_interval, flag, var, web_origin};
use crate::error::{register_secrets, scrub};
use crate::state::{AppState, Inner, Ports};

/// The deployed adapters: Solana (devnet, or localnet via `ARRHABON_CLUSTER`), Aurora (dry quotes unless `AURORA_LIVE=true`).
struct Live;

impl Ports for Live {
    type Ledger = SolanaLedger;
    type Quoter = AuroraQuoter;
    type Clock = ChainClock;
    type Attestor = KeypairAttestor;
    type Faucet = SolanaFaucet;

    async fn refresh_clock(ledger: &SolanaLedger, clock: &ChainClock) {
        if let Err(e) = clock.sync(ledger.chain()).await {
            tracing::warn!(error = %scrub(&e.to_string()), "chain clock sync failed");
        }
    }
}

const CHAIN_RETRY: Duration = Duration::from_secs(30);

/// Reads a project keypair (under `.keys/` by default); the key itself is never logged.
fn keypair(env: &str, default: &str) -> Result<Keypair, String> {
    let path = var(env).unwrap_or_else(|| default.to_owned());
    read_keypair_file(&path).map_err(|_| format!("{env}: cannot read keypair file {path}"))
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "arrhabon-api stopped");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let network = Network::from_env()?;
    let rpc_url = network.rpc_url();
    // Refuse a remote RPC for localnet before anything is sent to it.
    network.check_url(&rpc_url)?;
    // An RPC URL may carry an API key (Helius): keep it, and its query, out of every message.
    let query = rpc_url.split_once('?').map(|(_, q)| q.to_owned());
    register_secrets(
        [Some(rpc_url.clone()), query]
            .into_iter()
            .flatten()
            .collect(),
    );

    let crank_on = flag("CRANK", var("CRANK").as_deref(), true)?;
    let demo_on = flag("DEMO_ATTESTOR", var("DEMO_ATTESTOR").as_deref(), false)?;
    let faucet_on = flag("FAUCET", var("FAUCET").as_deref(), false)?;
    let crank_every = crank_interval(var("CRANK_INTERVAL_SECS").as_deref())?;
    let origin = web_origin(&var("WEB_ORIGIN").unwrap_or_else(|| "http://localhost:3000".into()))?;
    // With the crank off nothing is ever paid, so a throwaway key serves the reads.
    let crank_key = match crank_on {
        true => keypair("CRANK_KEYPAIR", ".keys/crank.json")?,
        false => Keypair::new(),
    };
    let attestor = match demo_on {
        true => {
            tracing::warn!("DEMO attestor enabled: /demo/attest signs with a demo-only key");
            Some(KeypairAttestor(keypair(
                "DEMO_ATTESTOR_KEYPAIR",
                ".keys/attestor.json",
            )?))
        }
        false => None,
    };
    let faucet_key = match faucet_on {
        true => {
            tracing::warn!("DEMO faucet enabled: /demo/faucet sends devnet SOL and test USDC");
            Some(keypair("FAUCET_KEYPAIR", ".keys/faucet.json")?)
        }
        false => None,
    };
    let quoter = AuroraQuoter::new(AuroraConfig::from_env())
        .map_err(|e| format!("aurora client: {}", scrub(&e.to_string())))?;
    let state: AppState<Live> = AppState(Arc::new(Inner {
        ledger: OnceLock::new(),
        quoter,
        clock: ChainClock::default(),
        attestor,
        crank: crank_on,
        faucet_on,
        faucet: OnceLock::new(),
        faucet_claims: Default::default(),
        cluster: network.name(),
    }));

    // A wrong cluster is fatal: the connect task reports it here and the server stops.
    let (fatal_tx, mut fatal) = watch::channel(None::<String>);
    let chain_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = connect_chain(chain_state, network, rpc_url, crank_key, faucet_key).await {
            fatal_tx.send_replace(Some(e));
        }
    });
    if crank_on {
        crank::spawn(state.clone(), crank_every);
    }

    let addr = format!(
        "{}:{}",
        var("HOST").unwrap_or_else(|| "127.0.0.1".into()),
        var("PORT").unwrap_or_else(|| "8080".into())
    );
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("binding {addr}: {e}"))?;
    tracing::info!(%addr, cluster = network.name(), crank = crank_on, demo_attestor = demo_on, "arrhabon-api listening");
    let mut fatal_seen = fatal.clone();
    axum::serve(listener, routes::router(state, origin))
        .with_graceful_shutdown(async move {
            tokio::select! {
                () = shutdown_signal() => {}
                // Errs (branch disabled) when the connect task ends without a fatal error.
                Ok(_) = fatal_seen.wait_for(Option::is_some) => {}
            }
        })
        .await
        .map_err(|e| e.to_string())?;
    match fatal.borrow_and_update().clone() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Resolves on Ctrl-C or SIGTERM (what container platforms send before killing).
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let term = async {
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "cannot listen for SIGTERM; Ctrl-C only");
                std::future::pending::<()>().await;
            }
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        () = term => {}
    }
    tracing::info!("shutdown signal received; draining requests");
}

/// Connects once the RPC passes the cluster guard and the program's Config exists;
/// retries until then. A refused cluster is an error and stops the server.
async fn connect_chain(
    state: AppState<Live>,
    network: Network,
    rpc_url: String,
    crank: Keypair,
    faucet_key: Option<Keypair>,
) -> Result<(), String> {
    loop {
        match ensure_cluster(network, &rpc_url).await {
            Err(PortError::Unavailable(e)) => tracing::warn!(
                error = %scrub(&e),
                "RPC unreachable; retrying in {}s", CHAIN_RETRY.as_secs()
            ),
            Err(e) => return Err(scrub(&e.to_string())),
            Ok(()) => {
                match SolanaLedger::connect(network, &rpc_url, crank.insecure_clone()).await {
                    Ok(ledger) => {
                        Live::refresh_clock(&ledger, &state.clock).await;
                        if let Some(key) = faucet_key.as_ref() {
                            let chain = Chain::new(&rpc_url, Arc::new(crank.insecure_clone()))
                                .map_err(|e| scrub(&e.to_string()))?;
                            let faucet =
                                SolanaFaucet::new(chain, key.insecure_clone(), ledger.mint());
                            tracing::info!(address = %faucet.address(), "DEMO faucet ready");
                            let _ = state.faucet.set(faucet);
                        }
                        let _ = state.ledger.set(ledger);
                        tracing::info!(
                            cluster = network.name(),
                            "chain ready: program Config found"
                        );
                        return Ok(());
                    }
                    Err(e) => tracing::warn!(
                        error = %scrub(&e.to_string()),
                        "chain not initialised; retrying in {}s", CHAIN_RETRY.as_secs()
                    ),
                }
            }
        }
        tokio::time::sleep(CHAIN_RETRY).await;
    }
}
