//! Wrappers for the admin instructions: Config and the university/attestor registry.

use arrhabon::state::{ATTESTOR_SEED, CONFIG_SEED, UNIVERSITY_SEED};
use solana_keypair::Keypair;

use super::{Env, Pubkey, key, program_data};

pub fn config_pda() -> Pubkey {
    Pubkey::find_program_address(&[CONFIG_SEED], &arrhabon::ID).0
}

pub fn university_pda(wallet: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[UNIVERSITY_SEED, wallet.as_ref()], &arrhabon::ID).0
}

pub fn attestor_pda(wallet: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[ATTESTOR_SEED, wallet.as_ref()], &arrhabon::ID).0
}

impl Env {
    pub fn init_config(&mut self, admin: &Keypair, mint: Pubkey) -> Result<(), String> {
        let accounts = arrhabon::accounts::InitConfig {
            admin: key(admin),
            config: config_pda(),
            mint,
            program_data: program_data(),
            system_program: anchor_lang::system_program::ID,
        };
        self.send(accounts, arrhabon::instruction::InitConfig {}, &[admin])
    }

    pub fn register_university(&mut self, admin: &Keypair, wallet: Pubkey) -> Result<(), String> {
        let accounts = arrhabon::accounts::RegisterUniversity {
            admin: key(admin),
            config: config_pda(),
            university: university_pda(&wallet),
            system_program: anchor_lang::system_program::ID,
        };
        let data = arrhabon::instruction::RegisterUniversity { wallet };
        self.send(accounts, data, &[admin])
    }

    pub fn register_attestor(&mut self, admin: &Keypair, wallet: Pubkey) -> Result<(), String> {
        let accounts = arrhabon::accounts::RegisterAttestor {
            admin: key(admin),
            config: config_pda(),
            attestor: attestor_pda(&wallet),
            system_program: anchor_lang::system_program::ID,
        };
        let data = arrhabon::instruction::RegisterAttestor { wallet };
        self.send(accounts, data, &[admin])
    }

    pub fn set_university_active(
        &mut self,
        admin: &Keypair,
        wallet: Pubkey,
        active: bool,
    ) -> Result<(), String> {
        let accounts = arrhabon::accounts::SetUniversityActive {
            admin: key(admin),
            config: config_pda(),
            university: university_pda(&wallet),
        };
        let data = arrhabon::instruction::SetUniversityActive { active };
        self.send(accounts, data, &[admin])
    }

    pub fn set_attestor_active(
        &mut self,
        admin: &Keypair,
        wallet: Pubkey,
        active: bool,
    ) -> Result<(), String> {
        let accounts = arrhabon::accounts::SetAttestorActive {
            admin: key(admin),
            config: config_pda(),
            attestor: attestor_pda(&wallet),
        };
        let data = arrhabon::instruction::SetAttestorActive { active };
        self.send(accounts, data, &[admin])
    }
}
