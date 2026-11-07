//! `register <university|attestor> <pubkey>`: the admin registers a university or attestor.

use std::str::FromStr;
use std::sync::Arc;

use anchor_client::Signer;
use anchor_lang::prelude::Pubkey;
use arrhabon::state::{Attestor, University};
use arrhabon_solana_adapter::instructions as ixs;
use arrhabon_solana_adapter::{Chain, Keypair, Network};

use crate::Res;

const USAGE: &str = "usage: devnet register <university|attestor> <PUBKEY>";

#[derive(Debug, PartialEq, Eq)]
pub enum Entry {
    Attestor,
    University,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Target {
    pub entry: Entry,
    pub wallet: Pubkey,
}

/// Parses the arguments after `register`.
pub fn parse(args: &[String]) -> Res<Target> {
    let [entry, wallet] = args else {
        return Err(USAGE.into());
    };
    let entry = match entry.as_str() {
        "attestor" => Entry::Attestor,
        "university" => Entry::University,
        _ => return Err(USAGE.into()),
    };
    let wallet = Pubkey::from_str(wallet).map_err(|e| format!("bad pubkey {wallet}: {e}"))?;
    Ok(Target { entry, wallet })
}

pub async fn run(t: &Target, admin: &Keypair, net: Network, url: &str) -> Res {
    let chain = Chain::new(url, Arc::new(admin.insecure_clone()))?;
    match t.entry {
        Entry::University => {
            let pda = ixs::university_pda(&t.wallet);
            if chain.fetch::<University>(pda).await?.is_some() {
                println!("university {} already registered (PDA {pda})", t.wallet);
                return Ok(());
            }
            let ix = ixs::register_university(admin.pubkey(), t.wallet);
            let sig = chain.send_as(&[ix], admin).await?;
            println!("admin      {}", admin.pubkey());
            println!("university {}", t.wallet);
            println!("signature  {}", sig.0);
            println!("explorer   {}", net.explorer_tx(&sig.0));
        }
        Entry::Attestor => {
            let pda = ixs::attestor_pda(&t.wallet);
            if chain.fetch::<Attestor>(pda).await?.is_some() {
                println!("attestor   {} already registered (PDA {pda})", t.wallet);
                return Ok(());
            }
            let ix = ixs::register_attestor(admin.pubkey(), t.wallet);
            let sig = chain.send_as(&[ix], admin).await?;
            println!("admin      {}", admin.pubkey());
            println!("attestor   {}", t.wallet);
            println!("signature  {}", sig.0);
            println!("explorer   {}", net.explorer_tx(&sig.0));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_both_entries() {
        let pk = Pubkey::new_unique();
        let t = parse(&args(&["attestor", &pk.to_string()])).unwrap();
        assert_eq!(
            t,
            Target {
                entry: Entry::Attestor,
                wallet: pk
            }
        );

        let t = parse(&args(&["university", &pk.to_string()])).unwrap();
        assert_eq!(
            t,
            Target {
                entry: Entry::University,
                wallet: pk
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        let pk = Pubkey::new_unique().to_string();
        for bad in [
            vec!["attestor"],
            vec!["vault", &pk],
            vec!["attestor", "not-a-key"],
            vec!["attestor", &pk, "extra"],
        ] {
            assert!(parse(&args(&bad)).is_err(), "expected error for {bad:?}");
        }
    }
}
