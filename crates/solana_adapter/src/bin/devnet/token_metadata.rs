//! `token-metadata --uri <url>`: Metaplex Token Metadata for the test mint, built by hand (Borsh).

use std::str::FromStr;
use std::sync::Arc;

use anchor_client::{Instruction, Signer};
use anchor_lang::prelude::{AccountMeta, Pubkey};
use anchor_lang::solana_program::system_program;
use arrhabon_solana_adapter::{Chain, Keypair, Network};

use crate::Res;

pub const PROGRAM: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const NAME: &str = "Arrhabon Test Dollar";
pub const SYMBOL: &str = "tUSD";
const CREATE_V3: u8 = 33;
const UPDATE_V2: u8 = 15;

#[derive(Debug, PartialEq)]
pub struct Fields {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

fn program() -> Pubkey {
    Pubkey::from_str(PROGRAM).expect("valid program id")
}

pub fn metadata_pda(mint: &Pubkey) -> Pubkey {
    let p = program();
    Pubkey::find_program_address(&[b"metadata", p.as_ref(), mint.as_ref()], &p).0
}

pub fn parse(args: &[String]) -> Res<String> {
    match args {
        [flag, uri] if flag == "--uri" && !uri.is_empty() => Ok(uri.clone()),
        _ => Err("usage: devnet token-metadata --uri <URL>".into()),
    }
}

fn string(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// DataV2: name, symbol, uri, seller_fee_basis_points = 0, creators/collection/uses = None.
fn data_v2(out: &mut Vec<u8>, f: &Fields) {
    string(out, &f.name);
    string(out, &f.symbol);
    string(out, &f.uri);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0]);
}

/// CreateMetadataAccountV3 { data: DataV2, is_mutable: true, collection_details: None }.
pub fn create_data(f: &Fields) -> Vec<u8> {
    let mut out = vec![CREATE_V3];
    data_v2(&mut out, f);
    out.extend_from_slice(&[1, 0]);
    out
}

/// UpdateMetadataAccountV2 { data: Some, new_update_authority/primary_sale/is_mutable: None }.
pub fn update_data(f: &Fields) -> Vec<u8> {
    let mut out = vec![UPDATE_V2, 1];
    data_v2(&mut out, f);
    out.extend_from_slice(&[0, 0, 0]);
    out
}

/// Accounts: metadata, mint, mint authority, payer, update authority, system program.
fn create_ix(mint: Pubkey, authority: Pubkey, f: &Fields) -> Instruction {
    Instruction {
        program_id: program(),
        accounts: vec![
            AccountMeta::new(metadata_pda(&mint), false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: create_data(f),
    }
}

fn update_ix(mint: Pubkey, authority: Pubkey, f: &Fields) -> Instruction {
    Instruction {
        program_id: program(),
        accounts: vec![
            AccountMeta::new(metadata_pda(&mint), false),
            AccountMeta::new_readonly(authority, true),
        ],
        data: update_data(f),
    }
}

/// Metadata account: key u8, update_authority, mint, then name/symbol/uri (stored NUL-padded).
pub fn decode(data: &[u8]) -> Option<Fields> {
    let mut at = 65;
    let mut next = || {
        let len = u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?) as usize;
        let s = std::str::from_utf8(data.get(at + 4..at + 4 + len)?).ok()?;
        at += 4 + len;
        Some(s.trim_end_matches('\0').to_string())
    };
    Some(Fields {
        name: next()?,
        symbol: next()?,
        uri: next()?,
    })
}

pub async fn run(uri: String, deployer: &Keypair, mint: Pubkey, net: Network, url: &str) -> Res {
    let chain = Chain::new(url, Arc::new(deployer.insecure_clone()))?;
    let want = Fields {
        name: NAME.into(),
        symbol: SYMBOL.into(),
        uri,
    };
    let pda = metadata_pda(&mint);
    let ix = match chain.account_data(&pda).await? {
        None => create_ix(mint, deployer.pubkey(), &want),
        Some(data) if decode(&data).as_ref() == Some(&want) => {
            println!("token metadata     {pda} already current");
            return Ok(());
        }
        Some(_) => update_ix(mint, deployer.pubkey(), &want),
    };
    let sig = chain.send_as(&[ix], deployer).await?;
    println!("token metadata     {}", net.explorer_tx(&sig.0));
    let data = chain.account_data(&pda).await?.ok_or("metadata missing")?;
    let got = decode(&data).ok_or("metadata undecodable")?;
    println!("  {} ({}) {}", got.name, got.symbol, got.uri);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> Fields {
        Fields {
            name: "Ab".into(),
            symbol: "C".into(),
            uri: "u".into(),
        }
    }

    #[test]
    fn create_matches_documented_layout() {
        let want: Vec<u8> = [
            &[33u8][..],               // CreateMetadataAccountV3
            &[2, 0, 0, 0, b'A', b'b'], // name
            &[1, 0, 0, 0, b'C'],       // symbol
            &[1, 0, 0, 0, b'u'],       // uri
            &[0, 0],                   // seller_fee_basis_points
            &[0, 0, 0],                // creators, collection, uses: None
            &[1],                      // is_mutable
            &[0],                      // collection_details: None
        ]
        .concat();
        assert_eq!(create_data(&fields()), want);
    }

    #[test]
    fn update_matches_documented_layout() {
        let want: Vec<u8> = [
            &[15u8, 1][..], // UpdateMetadataAccountV2, data: Some
            &[2, 0, 0, 0, b'A', b'b'],
            &[1, 0, 0, 0, b'C'],
            &[1, 0, 0, 0, b'u'],
            &[0, 0, 0, 0, 0],
            &[0, 0, 0], // new_update_authority, primary_sale_happened, is_mutable: None
        ]
        .concat();
        assert_eq!(update_data(&fields()), want);
    }

    #[test]
    fn decodes_padded_account() {
        let mut data = vec![4u8; 65];
        for (s, pad) in [("Ab", 32), ("C", 10), ("u", 200)] {
            string(&mut data, &format!("{s}{}", "\0".repeat(pad - s.len())));
        }
        data.extend_from_slice(&[0; 8]);
        assert_eq!(decode(&data), Some(fields()));
        assert_eq!(decode(&data[..70]), None);
    }

    #[test]
    fn parses_uri_flag() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            parse(&a(&["--uri", "http://x/t.json"])).unwrap(),
            "http://x/t.json"
        );
        assert!(parse(&a(&["--uri"])).is_err());
        assert!(parse(&a(&["http://x"])).is_err());
    }
}
