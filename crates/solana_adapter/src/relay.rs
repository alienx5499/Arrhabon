//! Relay guard: `submit` forwards only transactions that call this program.

use anchor_client::compat::solana_transaction::Transaction;
use anchor_lang::prelude::Pubkey;
use arrhabon_app::PortError;
use arrhabon_app::ports::PortResult;

/// Wallets may prepend compute-unit price and limit instructions.
const COMPUTE_BUDGET: Pubkey =
    Pubkey::from_str_const("ComputeBudget111111111111111111111111111111");

/// Every top-level instruction must target the Arrhabon program or ComputeBudget, and at
/// least one must target the program. The associated token program is only reached by CPI
/// from the program, so a top-level call to it is refused too.
pub fn check_programs(tx: &Transaction) -> PortResult<()> {
    let keys = &tx.message.account_keys;
    let mut calls_program = false;
    for ix in &tx.message.instructions {
        match keys.get(usize::from(ix.program_id_index)) {
            Some(id) if *id == arrhabon::ID => calls_program = true,
            Some(id) if *id == COMPUTE_BUDGET => {}
            Some(id) => {
                return Err(PortError::rejected(format!(
                    "instruction calls program {id}; only Arrhabon and ComputeBudget are relayed"
                )));
            }
            None => {
                return Err(PortError::rejected(
                    "instruction program index out of range",
                ));
            }
        }
    }
    match calls_program {
        true => Ok(()),
        false => Err(PortError::rejected(
            "transaction does not call the Arrhabon program",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::{pk, stored_vault};
    use crate::instructions::{contribute, vault_pda};
    use anchor_client::Instruction;
    use anchor_spl::associated_token;
    use arrhabon::state::Status;

    fn tx(ixs: &[Instruction]) -> Transaction {
        Transaction::new_with_payer(ixs, Some(&pk(5)))
    }

    fn other(program_id: Pubkey) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![],
            data: vec![],
        }
    }

    #[test]
    fn program_and_compute_budget_instructions_are_relayed() {
        let ix = contribute(vault_pda(&pk(1), 0), &stored_vault(Status::Open), pk(5), 1);
        assert_eq!(check_programs(&tx(std::slice::from_ref(&ix))), Ok(()));
        assert_eq!(check_programs(&tx(&[other(COMPUTE_BUDGET), ix])), Ok(()));
    }

    #[test]
    fn foreign_programs_and_bare_compute_budget_are_refused() {
        let ix = contribute(vault_pda(&pk(1), 0), &stored_vault(Status::Open), pk(5), 1);
        let system = anchor_lang::solana_program::system_program::ID;
        for foreign in [system, associated_token::ID, anchor_spl::token::ID] {
            let res = check_programs(&tx(&[ix.clone(), other(foreign)]));
            assert!(matches!(res, Err(PortError::Rejected { .. })), "{foreign}");
        }
        let res = check_programs(&tx(&[other(COMPUTE_BUDGET)]));
        assert!(matches!(res, Err(PortError::Rejected { .. })));
    }
}
