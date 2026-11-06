//! Commands: happy paths, pre-checks before any I/O, and error mapping.

use arrhabon_domain::{DomainError, VaultStatus};

use super::*;
use crate::commands::{AttestVisa, Contribute, CreateVault};
use crate::fakes::{FakeSigner, sign_as};
use crate::ports::{
    AttestorSigner, PortResult, SignedTx, TxSignature, UnsignedTx, VaultLedger, VisaOutcome,
};
use crate::{AppError, PortError};

fn create(deadline: i64, attestor: AccountId) -> CreateVault {
    CreateVault {
        student: STUDENT,
        university: UNIVERSITY,
        attestor,
        target: usdc(1_000),
        deadline: Deadline::new(deadline),
        nonce: 7,
    }
}

struct BrokenSigner;

impl AttestorSigner for BrokenSigner {
    fn attestor(&self) -> AccountId {
        ATTESTOR
    }

    async fn sign(&self, _tx: UnsignedTx) -> PortResult<SignedTx> {
        Err(PortError::Unavailable("hsm offline".into()))
    }
}

#[test]
fn create_vault_builds_a_transaction_the_student_signs() {
    let (clock, ledger) = setup();
    let built = run(create(NOW + 120, ATTESTOR).execute(&ledger, &clock)).unwrap();
    assert_eq!(built.vault, ledger.vault_address(STUDENT, 7));
    assert!(built.tx.last_valid_block_height > 0);
    run(ledger.submit(sign_as(built.tx, STUDENT))).unwrap();
    assert_eq!(
        ledger.stored_vault(built.vault).unwrap().status(),
        VaultStatus::Open
    );
}

#[test]
fn create_vault_rejects_bad_input_through_the_domain() {
    let (clock, ledger) = setup();
    let past = run(create(NOW, ATTESTOR).execute(&ledger, &clock));
    assert_eq!(past, Err(AppError::Conflict(DomainError::DeadlinePassed)));
    let self_attested = run(create(NOW + 120, STUDENT).execute(&ledger, &clock));
    assert_eq!(
        self_attested,
        Err(AppError::Invalid(DomainError::InvalidParties))
    );
    assert_eq!(ledger.queued(), 0);
}

#[test]
fn create_vault_rejects_unregistered_parties_before_building() {
    let (clock, ledger) = setup();
    let stranger = AccountId([9; 32]);
    let mut cmd = create(NOW + 120, ATTESTOR);
    cmd.university = stranger;
    assert_eq!(
        run(cmd.execute(&ledger, &clock)),
        Err(AppError::UniversityNotRegistered(stranger))
    );
    assert_eq!(
        run(create(NOW + 120, stranger).execute(&ledger, &clock)),
        Err(AppError::AttestorNotRegistered(stranger))
    );
    assert_eq!(ledger.queued(), 0);
}

#[test]
fn contribute_builds_then_records_on_submit() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 100)]);
    let cmd = Contribute {
        vault,
        funder: AUNT,
        amount: usdc(250),
    };
    let tx = run(cmd.execute(&ledger, &clock)).unwrap();
    run(ledger.submit(sign_as(tx, AUNT))).unwrap();
    assert_eq!(
        run(ledger.contribution(vault, AUNT))
            .unwrap()
            .unwrap()
            .amount(),
        usdc(350)
    );
}

#[test]
fn duplicate_submit_is_already_applied() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 120, &[]);
    let cmd = Contribute {
        vault,
        funder: AUNT,
        amount: usdc(250),
    };
    let signed = sign_as(run(cmd.execute(&ledger, &clock)).unwrap(), AUNT);
    run(ledger.submit(signed.clone())).unwrap();
    assert_eq!(run(ledger.submit(signed)), Err(PortError::AlreadyApplied));
    assert_eq!(
        ledger.stored_vault(vault).unwrap().total_contributed(),
        usdc(250)
    );
}

#[test]
fn contribute_failures() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 1, &[]);
    let attempt = |vault, funder, amount| {
        run(Contribute {
            vault,
            funder,
            amount,
        }
        .execute(&ledger, &clock))
    };

    assert_eq!(
        attempt(vault, AUNT, Usdc::new(10)),
        Err(AppError::Invalid(DomainError::BelowMinimum))
    );
    let missing = AccountId([9; 32]);
    assert_eq!(
        attempt(missing, AUNT, usdc(5)),
        Err(AppError::VaultNotFound(missing))
    );
    assert!(matches!(
        attempt(vault, AccountId([0; 32]), usdc(5)),
        Err(AppError::InvalidInput(_))
    ));
    clock.set(NOW + 1);
    assert_eq!(
        attempt(vault, AUNT, usdc(5)),
        Err(AppError::Conflict(DomainError::DeadlinePassed))
    );
}

#[test]
fn attest_issued_releases_a_funded_vault() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 600), (UNCLE, 400)]);
    let cmd = AttestVisa {
        vault,
        outcome: VisaOutcome::Issued,
    };
    let sig = run(cmd.execute(&ledger, &FakeSigner(ATTESTOR), &clock)).unwrap();
    assert_eq!(sig, TxSignature("sig-1".into()));
    assert_eq!(
        ledger.stored_vault(vault).unwrap().status(),
        VaultStatus::Closed
    );
}

#[test]
fn attest_failures_never_reach_the_chain() {
    let (clock, ledger) = setup();
    let short = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 999)]);
    let funded = seed_vault(&ledger, 2, NOW + 120, &[(AUNT, 1_000)]);
    let attest = |vault, outcome, signer| {
        run(AttestVisa { vault, outcome }.execute(&ledger, &FakeSigner(signer), &clock))
    };

    assert_eq!(
        attest(short, VisaOutcome::Issued, ATTESTOR),
        Err(AppError::Conflict(DomainError::BelowTarget))
    );
    assert_eq!(
        attest(funded, VisaOutcome::Issued, UNIVERSITY),
        Err(AppError::Forbidden(DomainError::Unauthorized))
    );
    clock.set(NOW + 120);
    assert_eq!(
        attest(funded, VisaOutcome::Issued, ATTESTOR),
        Err(AppError::Conflict(DomainError::DeadlinePassed))
    );
    assert_eq!(
        ledger.stored_vault(funded).unwrap().status(),
        VaultStatus::Open
    );
}

#[test]
fn attest_fails_cleanly_when_the_signer_fails() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 1_000)]);
    let cmd = AttestVisa {
        vault,
        outcome: VisaOutcome::Issued,
    };
    assert_eq!(
        run(cmd.execute(&ledger, &BrokenSigner, &clock)),
        Err(AppError::Unavailable("hsm offline".into()))
    );
    assert_eq!(
        ledger.stored_vault(vault).unwrap().status(),
        VaultStatus::Open
    );
}

#[test]
fn attest_denied_starts_refunds_even_after_the_deadline() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 100)]);
    clock.set(NOW + 500);
    let cmd = AttestVisa {
        vault,
        outcome: VisaOutcome::Denied,
    };
    run(cmd.execute(&ledger, &FakeSigner(ATTESTOR), &clock)).unwrap();
    assert_eq!(
        ledger.stored_vault(vault).unwrap().status(),
        VaultStatus::Refunding
    );
}

#[test]
fn chain_rejections_keep_their_program_error_class() {
    let rejected = |code| PortError::Rejected {
        code,
        message: "program error".into(),
    };
    let cases = [
        (
            rejected(Some(DomainError::Unauthorized)),
            AppError::Forbidden(DomainError::Unauthorized),
        ),
        (
            rejected(Some(DomainError::NotOpen)),
            AppError::Conflict(DomainError::NotOpen),
        ),
        (
            rejected(Some(DomainError::BelowMinimum)),
            AppError::Invalid(DomainError::BelowMinimum),
        ),
        (rejected(None), AppError::Rejected("program error".into())),
        (PortError::AlreadyApplied, AppError::AlreadyApplied),
        (
            PortError::Unavailable("rpc down".into()),
            AppError::Unavailable("rpc down".into()),
        ),
    ];
    for (port, app) in cases {
        assert_eq!(AppError::from(port), app);
    }
}

#[test]
fn a_chain_rejection_after_the_precheck_surfaces_its_code() {
    let (clock, ledger) = setup();
    let vault = seed_vault(&ledger, 1, NOW + 10, &[]);
    let cmd = Contribute {
        vault,
        funder: AUNT,
        amount: usdc(5),
    };
    let tx = run(cmd.execute(&ledger, &clock)).unwrap();
    clock.set(NOW + 10); // the deadline passes while the funder signs
    let err = run(ledger.submit(sign_as(tx, AUNT))).unwrap_err();
    assert_eq!(
        AppError::from(err),
        AppError::Conflict(DomainError::DeadlinePassed)
    );
}
