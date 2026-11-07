//! Crank commands: clock-skew grace, partial failures, idempotent reruns.

use arrhabon_domain::VaultStatus;

use super::*;
use crate::AppError;
use crate::commands::{Done, ExpireDue, Outcome, RefundKey, RunRefunds};
use crate::ports::TxSignature;

fn sent(n: u64) -> Result<Done, AppError> {
    Ok(Done::Submitted(TxSignature(format!("sig-{n}"))))
}

#[test]
fn expire_due_waits_out_the_grace_period_after_the_deadline() {
    let (clock, ledger) = setup();
    let due = seed_vault(&ledger, 1, DUE, &[(AUNT, 100)]);
    let not_yet = seed_vault(&ledger, 2, DUE + 1, &[(AUNT, 100)]);
    let outcomes = run(ExpireDue.execute(&ledger, &clock)).unwrap();
    assert_eq!(
        outcomes,
        vec![Outcome {
            item: due,
            result: sent(1)
        }]
    );
    assert_eq!(
        ledger.stored_vault(due).unwrap().status(),
        VaultStatus::Refunding
    );
    assert_eq!(
        ledger.stored_vault(not_yet).unwrap().status(),
        VaultStatus::Open
    );

    clock.set(NOW + 1);
    let outcomes = run(ExpireDue.execute(&ledger, &clock)).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].item, not_yet);
}

#[test]
fn expire_due_continues_past_a_failure() {
    let (clock, ledger) = setup();
    let broken = seed_vault(&ledger, 1, DUE, &[(AUNT, 100)]);
    let fine = seed_vault(&ledger, 2, DUE, &[]);
    ledger.fail_for(broken);
    let outcomes = run(ExpireDue.execute(&ledger, &clock)).unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(matches!(outcomes[0].result, Err(AppError::Unavailable(_))));
    assert!(outcomes[1].result.is_ok());
    assert_eq!(
        ledger.stored_vault(fine).unwrap().status(),
        VaultStatus::Closed
    ); // nobody funded
}

#[test]
fn expire_due_rerun_on_a_stale_read_counts_as_done() {
    let (clock, ledger) = setup();
    let due = seed_vault(&ledger, 1, DUE, &[(AUNT, 100)]);
    ledger.freeze_reads(); // the RPC keeps serving the pre-expiry state
    run(ExpireDue.execute(&ledger, &clock)).unwrap();
    let rerun = run(ExpireDue.execute(&ledger, &clock)).unwrap();
    assert_eq!(
        rerun,
        vec![Outcome {
            item: due,
            result: Ok(Done::AlreadyApplied)
        }]
    );
}

#[test]
fn run_refunds_collects_per_item_results_on_partial_failure() {
    let (clock, ledger) = setup();
    let a = seed_vault(&ledger, 1, DUE, &[(AUNT, 300), (UNCLE, 200)]);
    let b = seed_vault(&ledger, 2, DUE, &[(AUNT, 100)]);
    let open = seed_vault(&ledger, 3, NOW + 120, &[(AUNT, 100)]);
    run(ExpireDue.execute(&ledger, &clock)).unwrap();
    ledger.fail_for(UNCLE);

    let outcomes = run(RunRefunds.execute(&ledger)).unwrap();
    assert_eq!(outcomes.len(), 3);
    let failed: Vec<RefundKey> = outcomes
        .iter()
        .filter(|o| o.result.is_err())
        .map(|o| o.item)
        .collect();
    assert_eq!(
        failed,
        vec![RefundKey {
            vault: a,
            funder: UNCLE
        }]
    );
    assert_eq!(
        ledger.stored_vault(a).unwrap().status(),
        VaultStatus::Refunding
    );
    assert_eq!(
        ledger.stored_vault(b).unwrap().status(),
        VaultStatus::Closed
    );
    assert_eq!(
        ledger.stored_vault(open).unwrap().status(),
        VaultStatus::Open
    );

    // A second run retries only what is still owed.
    let retry = run(RunRefunds.execute(&ledger)).unwrap();
    assert_eq!(retry.iter().map(|o| o.item).collect::<Vec<_>>(), failed);
}

#[test]
fn run_refunds_rerun_on_a_stale_read_counts_as_done() {
    let (clock, ledger) = setup();
    seed_vault(&ledger, 1, DUE, &[(AUNT, 300), (UNCLE, 200)]);
    run(ExpireDue.execute(&ledger, &clock)).unwrap();
    ledger.freeze_reads();
    let first = run(RunRefunds.execute(&ledger)).unwrap();
    assert!(
        first
            .iter()
            .all(|o| matches!(o.result, Ok(Done::Submitted(_))))
    );
    let rerun = run(RunRefunds.execute(&ledger)).unwrap();
    assert_eq!(rerun.len(), 2);
    assert!(rerun.iter().all(|o| o.result == Ok(Done::AlreadyApplied)));
}

#[test]
fn run_refunds_leaves_funders_without_a_token_account_to_claim_themselves() {
    let (clock, ledger) = setup();
    let a = seed_vault(&ledger, 1, DUE, &[(AUNT, 300), (UNCLE, 200)]);
    run(ExpireDue.execute(&ledger, &clock)).unwrap();
    // Refunding UNCLE would make the crank pay rent for a new token account.
    ledger.without_token_account(UNCLE);

    let outcomes = run(RunRefunds.execute(&ledger)).unwrap();
    let key = |funder| RefundKey { vault: a, funder };
    assert_eq!(
        outcomes,
        vec![
            Outcome {
                item: key(AUNT),
                result: sent(2)
            },
            Outcome {
                item: key(UNCLE),
                result: Ok(Done::LeftForFunder)
            },
        ]
    );
    assert_eq!(
        ledger.stored_vault(a).unwrap().status(),
        VaultStatus::Refunding
    );
}
