//! `registry <attestor|university> <pubkey> --active=<true|false>`: the admin switches an entry.

use std::str::FromStr;
use std::sync::Arc;

use anchor_client::Signer;
use anchor_lang::prelude::Pubkey;
use arrhabon_solana_adapter::instructions as ixs;
use arrhabon_solana_adapter::{Chain, Keypair};

use crate::Res;

const USAGE: &str = "usage: devnet registry <attestor|university> <PUBKEY> --active=<true|false>";

#[derive(Debug, PartialEq)]
pub enum Entry {
    Attestor,
    University,
}

#[derive(Debug, PartialEq)]
pub struct Switch {
    pub entry: Entry,
    pub wallet: Pubkey,
    pub active: bool,
}

/// Parses the arguments after `registry`.
pub fn parse(args: &[String]) -> Res<Switch> {
    let [entry, wallet, flag] = args else {
        return Err(USAGE.into());
    };
    let entry = match entry.as_str() {
        "attestor" => Entry::Attestor,
        "university" => Entry::University,
        _ => return Err(USAGE.into()),
    };
    let wallet = Pubkey::from_str(wallet).map_err(|e| format!("bad pubkey {wallet}: {e}"))?;
    let active = match flag.as_str() {
        "--active=true" => true,
        "--active=false" => false,
        _ => return Err(USAGE.into()),
    };
    Ok(Switch {
        entry,
        wallet,
        active,
    })
}

pub async fn run(s: &Switch, admin: &Keypair, url: &str) -> Res {
    let chain = Chain::new(url, Arc::new(admin.insecure_clone()))?;
    let ix = match s.entry {
        Entry::Attestor => ixs::set_attestor_active(admin.pubkey(), s.wallet, s.active),
        Entry::University => ixs::set_university_active(admin.pubkey(), s.wallet, s.active),
    };
    let sig = chain.send_as(&[ix], admin).await?;
    println!("admin     {}", admin.pubkey());
    println!(
        "{:<9} {} active={}",
        format!("{:?}", s.entry).to_lowercase(),
        s.wallet,
        s.active
    );
    println!("signature {}", sig.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_both_entries_and_flags() {
        let pk = Pubkey::new_unique();
        let s = parse(&args(&["attestor", &pk.to_string(), "--active=false"])).unwrap();
        assert_eq!(
            s,
            Switch {
                entry: Entry::Attestor,
                wallet: pk,
                active: false
            }
        );
        let s = parse(&args(&["university", &pk.to_string(), "--active=true"])).unwrap();
        assert_eq!((s.entry, s.active), (Entry::University, true));
    }

    #[test]
    fn rejects_bad_input() {
        let pk = Pubkey::new_unique().to_string();
        for bad in [
            vec!["attestor", &pk],
            vec!["vault", &pk, "--active=true"],
            vec!["attestor", "not-a-key", "--active=true"],
            vec!["attestor", &pk, "--active=yes"],
            vec!["attestor", &pk, "--active"],
            vec!["attestor", &pk, "--active=true", "extra"],
        ] {
            assert!(parse(&args(&bad)).is_err(), "{bad:?}");
        }
    }
}
