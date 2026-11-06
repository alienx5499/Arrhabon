//! Views: vault countdown and contributors, funder refund state, university proof.

use arrhabon_domain::VaultStatus;

use super::*;
use crate::AppError;
use crate::address::{parse_account, to_base58};
use crate::commands::{ExpireDue, RunRefunds};
use crate::queries::{FunderView, RefundState, RegistryView, UniversityProof, VaultView};

#[test]
fn vault_view_shows_countdown_and_contributors() {
    let (clock, ledger) = setup();
    let id = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 300), (UNCLE, 200)]);
    let view = run(VaultView::load(&ledger, &clock, id)).unwrap();
    assert_eq!(view.status, VaultStatus::Open);
    assert_eq!(view.target, 1_000_000_000);
    assert_eq!(view.total_contributed, 500_000_000);
    assert_eq!(view.seconds_left, 120);
    assert_eq!(view.contributors.len(), 2);
    assert_eq!(view.vault, to_base58(id));
    assert_eq!(view.attestor, to_base58(ATTESTOR));

    clock.set(NOW + 500);
    assert_eq!(
        run(VaultView::load(&ledger, &clock, id))
            .unwrap()
            .seconds_left,
        0
    );
}

#[test]
fn vault_views_for_a_student_list_only_their_vaults() {
    let (clock, ledger) = setup();
    let a = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 300)]);
    let b = seed_vault(&ledger, 2, NOW + 120, &[]);
    let views = run(VaultView::for_student(&ledger, &clock, STUDENT)).unwrap();
    let ids: Vec<String> = views.iter().map(|v| v.vault.clone()).collect();
    assert_eq!(ids, vec![to_base58(a), to_base58(b)]);
    assert_eq!(views[0].contributors.len(), 1);
    assert!(
        run(VaultView::for_student(&ledger, &clock, AUNT))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn vault_view_of_unknown_vault_is_not_found() {
    let (clock, ledger) = setup();
    let missing = AccountId([9; 32]);
    assert_eq!(
        run(VaultView::load(&ledger, &clock, missing)),
        Err(AppError::VaultNotFound(missing))
    );
}

#[test]
fn funder_view_tracks_refund_state() {
    let (clock, ledger) = setup();
    let open = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 100)]);
    let expired = seed_vault(&ledger, 2, DUE, &[(AUNT, 50), (UNCLE, 50)]);
    run(ExpireDue.execute(&ledger, &clock)).unwrap();

    let state = |vault| {
        let view = run(FunderView::load(&ledger, AUNT)).unwrap();
        view.contributions
            .iter()
            .find(|c| c.vault == to_base58(vault))
            .unwrap()
            .refund
    };
    assert_eq!(state(open), RefundState::NotDue);
    assert_eq!(state(expired), RefundState::Pending);

    ledger.fail_for(UNCLE); // the uncle's refund fails, so the vault stays Refunding
    run(RunRefunds.execute(&ledger)).unwrap();
    assert_eq!(state(expired), RefundState::Refunded);
}

#[test]
fn funder_view_marks_a_missing_vault_unavailable_instead_of_failing() {
    let (_, ledger) = setup();
    let open = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 100)]);
    let gone = AccountId([0x77; 32]);
    ledger.put_contribution(Contribution::from_parts(gone, AUNT, usdc(5), false).unwrap());

    let view = run(FunderView::load(&ledger, AUNT)).unwrap();
    let entry = |vault| {
        view.contributions
            .iter()
            .find(|c| c.vault == to_base58(vault))
            .unwrap()
    };
    assert_eq!(entry(gone).vault_status, None);
    assert_eq!(entry(gone).refund, RefundState::Unavailable);
    assert_eq!(entry(open).vault_status, Some(VaultStatus::Open));
}

#[test]
fn university_proof_counts_only_unrefunded_funds() {
    let (clock, ledger) = setup();
    let id = seed_vault(&ledger, 1, DUE, &[(AUNT, 600), (UNCLE, 400)]);
    run(ExpireDue.execute(&ledger, &clock)).unwrap();
    ledger.fail_for(UNCLE);
    run(RunRefunds.execute(&ledger)).unwrap();
    let proof = run(UniversityProof::for_vault(&ledger, id)).unwrap();
    assert_eq!(proof.status, VaultStatus::Refunding);
    assert_eq!(proof.funds_held, 400_000_000);
}

#[test]
fn university_proof_shows_funds_held_and_attestor() {
    let (_, ledger) = setup();
    let id = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 600), (UNCLE, 400)]);
    let proof = run(UniversityProof::for_vault(&ledger, id)).unwrap();
    assert_eq!(proof.funds_held, 1_000_000_000);
    assert_eq!(proof.attestor, to_base58(ATTESTOR));
    assert!(proof.release_condition.contains(&to_base58(ATTESTOR)));

    let all = run(UniversityProof::for_university(&ledger, UNIVERSITY)).unwrap();
    assert_eq!(all, vec![proof]);
}

#[test]
fn addresses_round_trip_and_reject_garbage() {
    assert_eq!(parse_account(&to_base58(AUNT)), Ok(AUNT));
    for bad in ["", "0OIl", "abc", &to_base58(AccountId([0; 32]))] {
        assert!(
            matches!(parse_account(bad), Err(AppError::InvalidInput(_))),
            "{bad}"
        );
    }
    let too_long = bs58::encode([7u8; 33]).into_string();
    assert!(parse_account(&too_long).is_err());
}

#[test]
fn registry_view_lists_registered_parties() {
    let (_, ledger) = setup();
    ledger.register_university(AUNT);
    let view = run(RegistryView::load(&ledger)).unwrap();
    assert_eq!(
        view.universities,
        vec![to_base58(UNIVERSITY), to_base58(AUNT)]
    );
    assert_eq!(view.attestors, vec![to_base58(ATTESTOR)]);
}
