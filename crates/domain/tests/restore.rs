mod common;

use arrhabon_domain::{
    AccountId, Contribution, Deadline, DomainError, Usdc, Vault, VaultParts, VaultStatus,
};
use common::*;

/// A Refunding vault: MOM 600 refunded, UNCLE 400 still owed.
fn parts() -> VaultParts {
    VaultParts {
        id: VAULT,
        student: STUDENT,
        university: UNIVERSITY,
        attestor: ATTESTOR,
        target: usdc(1_000),
        deadline: Deadline::new(DEADLINE),
        status: VaultStatus::Refunding,
        total_contributed: usdc(1_000),
        refunded_total: usdc(600),
        contributor_count: 2,
        refunded_count: 1,
        has_outside_contributions: true,
    }
}

fn corrupt(p: VaultParts) {
    assert_eq!(Vault::from_parts(p), Err(DomainError::CorruptState));
}

#[test]
fn from_parts_loads_consistent_state() {
    let v = Vault::from_parts(parts()).unwrap();
    assert_eq!(v.id(), VAULT);
    assert_eq!(v.student(), STUDENT);
    assert_eq!(v.university(), UNIVERSITY);
    assert_eq!(v.attestor(), ATTESTOR);
    assert_eq!(v.target(), usdc(1_000));
    assert_eq!(v.deadline(), Deadline::new(DEADLINE));
    assert_eq!(v.status(), VaultStatus::Refunding);
    assert_eq!(v.refunded_total(), usdc(600));
    assert_eq!(v.refunded_count(), 1);
    assert!(v.has_outside_contributions());
}

#[test]
fn from_parts_rejects_more_refunds_than_contributors() {
    corrupt(VaultParts {
        refunded_count: 3,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_refunded_total_above_contributed() {
    corrupt(VaultParts {
        refunded_total: usdc(1_001),
        ..parts()
    });
}

#[test]
fn from_parts_rejects_released_with_refunds() {
    corrupt(VaultParts {
        status: VaultStatus::Released,
        ..parts()
    });
    corrupt(VaultParts {
        status: VaultStatus::Released,
        refunded_count: 0,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_released_below_target() {
    corrupt(VaultParts {
        status: VaultStatus::Released,
        total_contributed: usdc(999),
        refunded_total: Usdc::ZERO,
        refunded_count: 0,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_open_with_refunds() {
    corrupt(VaultParts {
        status: VaultStatus::Open,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_closed_with_partial_refunds() {
    corrupt(VaultParts {
        status: VaultStatus::Closed,
        ..parts()
    });
    corrupt(VaultParts {
        status: VaultStatus::Closed,
        refunded_count: 2,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_closed_below_target_without_refunds() {
    corrupt(VaultParts {
        status: VaultStatus::Closed,
        total_contributed: usdc(999),
        refunded_total: Usdc::ZERO,
        refunded_count: 0,
        ..parts()
    });
}

#[test]
fn from_parts_accepts_both_closed_paths() {
    let refunded = VaultParts {
        status: VaultStatus::Closed,
        refunded_total: usdc(1_000),
        refunded_count: 2,
        ..parts()
    };
    let released = VaultParts {
        status: VaultStatus::Closed,
        refunded_total: Usdc::ZERO,
        refunded_count: 0,
        ..parts()
    };
    assert!(Vault::from_parts(refunded).is_ok());
    assert!(Vault::from_parts(released).is_ok());
}

#[test]
fn from_parts_rejects_refunding_when_everyone_is_refunded() {
    corrupt(VaultParts {
        refunded_total: usdc(1_000),
        refunded_count: 2,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_counts_that_disagree_with_totals() {
    corrupt(VaultParts {
        status: VaultStatus::Open,
        contributor_count: 0,
        refunded_total: Usdc::ZERO,
        refunded_count: 0,
        ..parts()
    });
    corrupt(VaultParts {
        refunded_count: 0,
        ..parts()
    });
}

#[test]
fn from_parts_rejects_zero_target_or_nonpositive_deadline() {
    corrupt(VaultParts {
        target: Usdc::ZERO,
        ..parts()
    });
    corrupt(VaultParts {
        deadline: Deadline::new(0),
        ..parts()
    });
}

#[test]
fn from_parts_rejects_invalid_parties() {
    let r = Vault::from_parts(VaultParts {
        attestor: STUDENT,
        ..parts()
    });
    assert_eq!(r, Err(DomainError::InvalidParties));
    let r = Vault::from_parts(VaultParts {
        id: AccountId([0; 32]),
        ..parts()
    });
    assert_eq!(r, Err(DomainError::InvalidParties));
}

#[test]
fn contribution_from_parts_loads_and_validates() {
    let c = Contribution::from_parts(VAULT, MOM, usdc(600), true).unwrap();
    assert_eq!((c.vault(), c.funder(), c.amount()), (VAULT, MOM, usdc(600)));
    assert!(c.refunded());
    assert_eq!(
        Contribution::from_parts(VAULT, MOM, Usdc::ZERO, true),
        Err(DomainError::CorruptState)
    );
    assert_eq!(
        Contribution::from_parts(AccountId([0; 32]), MOM, usdc(1), false),
        Err(DomainError::InvalidParties)
    );
    assert_eq!(
        Contribution::from_parts(VAULT, AccountId([0; 32]), usdc(1), false),
        Err(DomainError::InvalidParties)
    );
}
