//! Program account layouts: decoding to domain types and memcmp filters on their fields.

use anchor_client::RpcFilterType;
use anchor_client::compat::solana_rpc_client_api::filter::Memcmp;
use anchor_lang::prelude::Pubkey;
use arrhabon::state::{self, Status, id};
use arrhabon_app::PortError;
use arrhabon_domain::{Contribution, Deadline, DomainError, Usdc, Vault, VaultParts};

/// Byte offsets after the 8-byte discriminator, in `state::Vault` field order.
pub const VAULT_STUDENT: usize = 8;
pub const VAULT_UNIVERSITY: usize = 40;
pub const VAULT_STATUS: usize = 160;
/// Byte offsets in `state::Contribution`.
pub const CONTRIBUTION_VAULT: usize = 8;
pub const CONTRIBUTION_FUNDER: usize = 40;
pub const CONTRIBUTION_REFUNDED: usize = 80;
/// Byte offset of `active` in `state::University` and `state::Attestor` (after `wallet`).
pub const REGISTRY_ACTIVE: usize = 40;

pub fn key_filter(offset: usize, key: &Pubkey) -> RpcFilterType {
    RpcFilterType::Memcmp(Memcmp::new_base58_encoded(offset, key.as_ref()))
}

pub fn status_filter(status: Status) -> RpcFilterType {
    RpcFilterType::Memcmp(Memcmp::new_base58_encoded(VAULT_STATUS, &[status as u8]))
}

pub fn unrefunded_filter() -> RpcFilterType {
    RpcFilterType::Memcmp(Memcmp::new_base58_encoded(CONTRIBUTION_REFUNDED, &[0]))
}

pub fn active_filter() -> RpcFilterType {
    RpcFilterType::Memcmp(Memcmp::new_base58_encoded(REGISTRY_ACTIVE, &[1]))
}

fn corrupt(e: DomainError) -> PortError {
    PortError::Rejected {
        code: Some(e),
        message: "stored account fails domain checks".into(),
    }
}

pub fn vault(key: &Pubkey, v: &state::Vault) -> Result<Vault, PortError> {
    Vault::from_parts(VaultParts {
        id: id(key),
        student: id(&v.student),
        university: id(&v.university),
        attestor: id(&v.attestor),
        target: Usdc::new(v.target),
        deadline: Deadline::new(v.deadline),
        status: v.status.into(),
        total_contributed: Usdc::new(v.total_contributed),
        refunded_total: Usdc::new(v.refunded_total),
        contributor_count: v.contributor_count,
        refunded_count: v.refunded_count,
        has_outside_contributions: v.has_outside_contributions,
    })
    .map_err(corrupt)
}

pub fn contribution(c: &state::Contribution) -> Result<Contribution, PortError> {
    Contribution::from_parts(id(&c.vault), id(&c.funder), Usdc::new(c.amount), c.refunded)
        .map_err(corrupt)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use anchor_lang::AccountSerialize;
    use arrhabon_domain::VaultStatus;

    pub fn pk(b: u8) -> Pubkey {
        Pubkey::new_from_array([b; 32])
    }

    pub fn stored_vault(status: Status) -> state::Vault {
        state::Vault {
            student: pk(1),
            university: pk(2),
            attestor: pk(3),
            mint: pk(4),
            nonce: 7,
            target: 1_000_000_000,
            deadline: 1_700_000_000,
            status,
            total_contributed: 0,
            refunded_total: 0,
            contributor_count: 0,
            refunded_count: 0,
            has_outside_contributions: false,
            bump: 255,
        }
    }

    fn bytes(a: &impl AccountSerialize) -> Vec<u8> {
        let mut out = Vec::new();
        a.try_serialize(&mut out).unwrap();
        out
    }

    #[test]
    fn vault_filter_offsets_match_the_account_layout() {
        let data = bytes(&stored_vault(Status::Refunding));
        assert_eq!(&data[VAULT_STUDENT..VAULT_STUDENT + 32], pk(1).as_ref());
        assert_eq!(
            &data[VAULT_UNIVERSITY..VAULT_UNIVERSITY + 32],
            pk(2).as_ref()
        );
        assert_eq!(data[VAULT_STATUS], Status::Refunding as u8);
        assert_eq!(Status::Open as u8, 0);
    }

    #[test]
    fn contribution_filter_offsets_match_the_account_layout() {
        let c = state::Contribution {
            vault: pk(5),
            funder: pk(6),
            amount: 9,
            refunded: true,
            bump: 1,
        };
        let data = bytes(&c);
        assert_eq!(
            &data[CONTRIBUTION_VAULT..CONTRIBUTION_VAULT + 32],
            pk(5).as_ref()
        );
        assert_eq!(
            &data[CONTRIBUTION_FUNDER..CONTRIBUTION_FUNDER + 32],
            pk(6).as_ref()
        );
        assert_eq!(data[CONTRIBUTION_REFUNDED], 1);
    }

    #[test]
    fn registry_active_offset_matches_both_layouts() {
        let u = state::University {
            wallet: pk(7),
            active: true,
            bump: 1,
        };
        let a = state::Attestor {
            wallet: pk(8),
            active: true,
            bump: 1,
        };
        for data in [bytes(&u), bytes(&a)] {
            assert_eq!(data[REGISTRY_ACTIVE], 1);
        }
    }

    #[test]
    fn decodes_a_stored_vault() {
        let v = vault(&pk(9), &stored_vault(Status::Open)).unwrap();
        assert_eq!(v.status(), VaultStatus::Open);
        assert_eq!(v.target(), Usdc::new(1_000_000_000));
        assert_eq!(v.id(), id(&pk(9)));
    }

    #[test]
    fn corrupt_vault_is_rejected_with_its_rule() {
        let mut s = stored_vault(Status::Open);
        s.total_contributed = 5; // no contributor recorded
        let e = vault(&pk(9), &s).unwrap_err();
        assert!(matches!(
            e,
            PortError::Rejected {
                code: Some(DomainError::CorruptState),
                ..
            }
        ));
    }

    #[test]
    fn decodes_a_contribution() {
        let c = state::Contribution {
            vault: pk(5),
            funder: pk(6),
            amount: 2_000_000,
            refunded: false,
            bump: 1,
        };
        let d = contribution(&c).unwrap();
        assert_eq!(d.funder(), id(&pk(6)));
        assert_eq!(d.amount(), Usdc::new(2_000_000));
    }
}
