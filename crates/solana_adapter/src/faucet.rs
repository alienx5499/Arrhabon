//! DEMO ONLY: `TestFunds` from a devnet faucet wallet that holds SOL and test USDC.

use anchor_client::Instruction;
use anchor_lang::prelude::{AccountMeta, Pubkey};
use anchor_lang::solana_program::system_program;
use anchor_spl::associated_token::{self, get_associated_token_address};
use anchor_spl::token;
use arrhabon_app::PortError;
use arrhabon_app::ports::{PortResult, TestFunds, TxSignature, WalletFunds};
use arrhabon_domain::{AccountId, Usdc};
use solana_keypair::Keypair;

use crate::Chain;
use crate::ledger::pk;

/// The test mint's decimals, as `transfer_checked` requires.
const USDC_DECIMALS: u8 = 6;
/// SPL token account layout: mint (32), owner (32), then the amount as a little-endian u64.
const TOKEN_AMOUNT_OFFSET: usize = 64;

pub struct SolanaFaucet {
    chain: Chain,
    wallet: Keypair,
    mint: Pubkey,
}

impl SolanaFaucet {
    pub fn new(chain: Chain, wallet: Keypair, mint: Pubkey) -> Self {
        SolanaFaucet {
            chain,
            wallet,
            mint,
        }
    }

    /// The faucet's own address, to fund it once with SOL and test USDC.
    pub fn address(&self) -> Pubkey {
        use anchor_client::Signer;
        self.wallet.pubkey()
    }

    async fn usdc_of(&self, owner: &Pubkey) -> PortResult<Usdc> {
        let ata = get_associated_token_address(owner, &self.mint);
        Ok(Usdc::new(
            self.chain
                .account_data(&ata)
                .await?
                .and_then(|d| token_amount(&d))
                .unwrap_or(0),
        ))
    }
}

impl TestFunds for SolanaFaucet {
    async fn funds(&self, wallet: AccountId) -> PortResult<WalletFunds> {
        let owner = pk(wallet);
        Ok(WalletFunds {
            lamports: self.chain.lamports(&owner).await?,
            usdc: self.usdc_of(&owner).await?,
        })
    }

    async fn send(&self, wallet: AccountId, lamports: u64, usdc: Usdc) -> PortResult<TxSignature> {
        if lamports == 0 && usdc.is_zero() {
            return Err(PortError::rejected("faucet send needs SOL or USDC"));
        }
        let ixs = grant_instructions(&self.address(), &pk(wallet), &self.mint, lamports, usdc);
        self.chain.send_as(&ixs, &self.wallet).await
    }
}

/// System transfer of SOL, then (if any USDC) create the recipient's token account if missing
/// and `transfer_checked` from the faucet's token account.
pub fn grant_instructions(
    faucet: &Pubkey,
    to: &Pubkey,
    mint: &Pubkey,
    lamports: u64,
    usdc: Usdc,
) -> Vec<Instruction> {
    let mut ixs = Vec::new();
    if lamports > 0 {
        let mut data = 2u32.to_le_bytes().to_vec();
        data.extend_from_slice(&lamports.to_le_bytes());
        ixs.push(Instruction {
            program_id: system_program::ID,
            accounts: vec![
                AccountMeta::new(*faucet, true),
                AccountMeta::new(*to, false),
            ],
            data,
        });
    }
    if !usdc.is_zero() {
        let to_ata = get_associated_token_address(to, mint);
        ixs.push(Instruction {
            program_id: associated_token::ID,
            accounts: vec![
                AccountMeta::new(*faucet, true),
                AccountMeta::new(to_ata, false),
                AccountMeta::new_readonly(*to, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token::ID, false),
            ],
            data: vec![1],
        });
        let mut data = vec![12];
        data.extend_from_slice(&usdc.micro().to_le_bytes());
        data.push(USDC_DECIMALS);
        ixs.push(Instruction {
            program_id: token::ID,
            accounts: vec![
                AccountMeta::new(get_associated_token_address(faucet, mint), false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(to_ata, false),
                AccountMeta::new_readonly(*faucet, true),
            ],
            data,
        });
    }
    ixs
}

fn token_amount(data: &[u8]) -> Option<u64> {
    let bytes = data.get(TOKEN_AMOUNT_OFFSET..TOKEN_AMOUNT_OFFSET + 8)?;
    Some(u64::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> Pubkey {
        Pubkey::new_from_array([n; 32])
    }

    #[test]
    fn sol_and_usdc_give_transfer_create_ata_and_transfer_checked() {
        let ixs = grant_instructions(&key(1), &key(2), &key(3), 50_000_000, Usdc::new(7));
        assert_eq!(ixs.len(), 3);
        assert_eq!(ixs[0].program_id, system_program::ID);
        assert_eq!(ixs[0].data[..4], 2u32.to_le_bytes());
        assert_eq!(ixs[0].data[4..], 50_000_000u64.to_le_bytes());
        assert_eq!(ixs[1].program_id, associated_token::ID);
        assert_eq!(ixs[1].data, vec![1], "idempotent create");
        assert_eq!(ixs[2].program_id, token::ID);
        assert_eq!(ixs[2].data[0], 12, "transfer_checked");
        assert_eq!(ixs[2].data[1..9], 7u64.to_le_bytes());
        assert_eq!(ixs[2].data[9], USDC_DECIMALS);
        assert!(
            ixs[2].accounts[3].is_signer,
            "the faucet signs the token transfer"
        );
    }

    #[test]
    fn zero_parts_are_left_out() {
        assert_eq!(
            grant_instructions(&key(1), &key(2), &key(3), 0, Usdc::new(5)).len(),
            2
        );
        assert_eq!(
            grant_instructions(&key(1), &key(2), &key(3), 9, Usdc::new(0)).len(),
            1
        );
    }

    #[test]
    fn token_amount_reads_the_layout_offset() {
        let mut data = vec![0u8; 165];
        data[64..72].copy_from_slice(&1_000u64.to_le_bytes());
        assert_eq!(token_amount(&data), Some(1_000));
        assert_eq!(token_amount(&[0u8; 10]), None);
    }
}
