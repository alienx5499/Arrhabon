//! Program instructions and PDAs, plus unsigned legacy transactions for a wallet to sign.

use anchor_client::Instruction;
use anchor_client::compat::solana_transaction::{Hash, Transaction};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::{bpf_loader_upgradeable, system_program};
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::{self, get_associated_token_address};
use anchor_spl::token;
use arrhabon::state::{ATTESTOR_SEED, CONFIG_SEED, CONTRIBUTION_SEED, UNIVERSITY_SEED, VAULT_SEED};
use arrhabon::{ID, accounts as acc, instruction as ix};

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &ID).0
}

pub fn vault_pda(student: &Pubkey, nonce: u64) -> Pubkey {
    pda(&[VAULT_SEED, student.as_ref(), &nonce.to_le_bytes()])
}

pub fn contribution_pda(vault: &Pubkey, funder: &Pubkey) -> Pubkey {
    pda(&[CONTRIBUTION_SEED, vault.as_ref(), funder.as_ref()])
}

pub fn config_pda() -> Pubkey {
    pda(&[CONFIG_SEED])
}

pub fn university_pda(wallet: &Pubkey) -> Pubkey {
    pda(&[UNIVERSITY_SEED, wallet.as_ref()])
}

pub fn attestor_pda(wallet: &Pubkey) -> Pubkey {
    pda(&[ATTESTOR_SEED, wallet.as_ref()])
}

fn build(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

/// `create_vault` arguments as the program takes them.
#[derive(Debug, Clone, Copy)]
pub struct VaultTerms {
    pub student: Pubkey,
    pub nonce: u64,
    pub university: Pubkey,
    pub attestor: Pubkey,
    pub target: u64,
    pub deadline: i64,
}

pub fn create_vault(t: VaultTerms, mint: Pubkey) -> Instruction {
    let vault = vault_pda(&t.student, t.nonce);
    build(
        acc::CreateVault {
            student: t.student,
            vault,
            config: config_pda(),
            registered_university: university_pda(&t.university),
            registered_attestor: attestor_pda(&t.attestor),
            mint,
            vault_token: get_associated_token_address(&vault, &mint),
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        },
        ix::CreateVault {
            nonce: t.nonce,
            university: t.university,
            attestor: t.attestor,
            target: t.target,
            deadline: t.deadline,
        },
    )
}

pub fn contribute(vault: Pubkey, funder: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    build(
        acc::Contribute {
            funder,
            vault,
            contribution: contribution_pda(&vault, &funder),
            mint,
            funder_token: get_associated_token_address(&funder, &mint),
            vault_token: get_associated_token_address(&vault, &mint),
            token_program: token::ID,
            system_program: system_program::ID,
        },
        ix::Contribute { amount },
    )
}

pub fn confirm_visa(
    vault: Pubkey,
    attestor: Pubkey,
    university: Pubkey,
    mint: Pubkey,
) -> Instruction {
    build(
        acc::ConfirmVisa {
            attestor,
            vault,
            university,
            mint,
            vault_token: get_associated_token_address(&vault, &mint),
            university_token: get_associated_token_address(&university, &mint),
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        },
        ix::ConfirmVisa {},
    )
}

pub fn deny_visa(vault: Pubkey, attestor: Pubkey) -> Instruction {
    build(acc::DenyVisa { attestor, vault }, ix::DenyVisa {})
}

pub fn expire(vault: Pubkey) -> Instruction {
    build(acc::Expire { vault }, ix::Expire {})
}

pub fn refund(vault: Pubkey, funder: Pubkey, caller: Pubkey, mint: Pubkey) -> Instruction {
    build(
        acc::Refund {
            caller,
            vault,
            contribution: contribution_pda(&vault, &funder),
            funder,
            mint,
            vault_token: get_associated_token_address(&vault, &mint),
            funder_token: get_associated_token_address(&funder, &mint),
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        },
        ix::Refund {},
    )
}

pub fn init_config(admin: Pubkey, mint: Pubkey) -> Instruction {
    let program_data = Pubkey::find_program_address(&[ID.as_ref()], &bpf_loader_upgradeable::ID).0;
    build(
        acc::InitConfig {
            admin,
            config: config_pda(),
            mint,
            program_data,
            system_program: system_program::ID,
        },
        ix::InitConfig {},
    )
}

pub fn register_university(admin: Pubkey, wallet: Pubkey) -> Instruction {
    build(
        acc::RegisterUniversity {
            admin,
            config: config_pda(),
            university: university_pda(&wallet),
            system_program: system_program::ID,
        },
        ix::RegisterUniversity { wallet },
    )
}

pub fn register_attestor(admin: Pubkey, wallet: Pubkey) -> Instruction {
    build(
        acc::RegisterAttestor {
            admin,
            config: config_pda(),
            attestor: attestor_pda(&wallet),
            system_program: system_program::ID,
        },
        ix::RegisterAttestor { wallet },
    )
}

pub fn set_university_active(admin: Pubkey, wallet: Pubkey, active: bool) -> Instruction {
    build(
        acc::SetUniversityActive {
            admin,
            config: config_pda(),
            university: university_pda(&wallet),
        },
        ix::SetUniversityActive { active },
    )
}

pub fn set_attestor_active(admin: Pubkey, wallet: Pubkey, active: bool) -> Instruction {
    build(
        acc::SetAttestorActive {
            admin,
            config: config_pda(),
            attestor: attestor_pda(&wallet),
        },
        ix::SetAttestorActive { active },
    )
}

/// Legacy transaction with empty signature slots, bincode-serialized; `payer` pays the fee.
pub fn unsigned_tx(ixs: &[Instruction], payer: &Pubkey, blockhash: Hash) -> Vec<u8> {
    let mut tx = Transaction::new_with_payer(ixs, Some(payer));
    tx.message.recent_blockhash = blockhash;
    bincode::serialize(&tx).expect("a Transaction always serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::pk;
    use anchor_lang::Discriminator;

    #[test]
    fn unsigned_create_vault_is_paid_and_signed_by_the_student() {
        let terms = VaultTerms {
            student: pk(1),
            nonce: 3,
            university: pk(2),
            attestor: pk(3),
            target: 1_000_000_000,
            deadline: 1_700_000_000,
        };
        let hash = Hash::new_from_array([7; 32]);
        let bytes = unsigned_tx(&[create_vault(terms, pk(4))], &pk(1), hash);
        let tx: Transaction = bincode::deserialize(&bytes).unwrap();

        assert_eq!(tx.message.account_keys[0], pk(1), "fee payer first");
        assert_eq!(tx.message.header.num_required_signatures, 1);
        assert_eq!(tx.signatures.len(), 1);
        assert!(!tx.is_signed());
        assert_eq!(tx.message.recent_blockhash, hash);
        let data = &tx.message.instructions[0].data;
        assert_eq!(&data[..8], ix::CreateVault::DISCRIMINATOR);
        assert!(tx.message.account_keys.contains(&vault_pda(&pk(1), 3)));
    }

    #[test]
    fn contribute_targets_the_funders_contribution_pda() {
        let vault = vault_pda(&pk(1), 0);
        let i = contribute(vault, pk(5), pk(4), 2_000_000);
        assert_eq!(i.accounts[0].pubkey, pk(5));
        assert!(i.accounts[0].is_signer);
        assert_eq!(i.accounts[2].pubkey, contribution_pda(&vault, &pk(5)));
        assert_eq!(&i.data[8..], &2_000_000u64.to_le_bytes());
    }

    #[test]
    fn vault_pda_depends_on_nonce() {
        assert_ne!(vault_pda(&pk(1), 0), vault_pda(&pk(1), 1));
    }
}
