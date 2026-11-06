mod common;

use arrhabon_domain::{
    Contribution, Deadline, DomainError, MIN_CONTRIBUTION, Usdc, Vault, VaultStatus,
};
use common::*;
use proptest::prelude::*;

fn funder(i: usize) -> arrhabon_domain::AccountId {
    let mut id = [0xAA; 32];
    id[..8].copy_from_slice(&(i as u64).to_le_bytes());
    arrhabon_domain::AccountId(id)
}

fn vault_with(amounts: &[u64]) -> (Vault, Vec<Contribution>) {
    let mut v = Vault::create(
        VAULT,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        Usdc::new(1),
        Deadline::new(100),
        0,
    )
    .unwrap();
    let mut cs: Vec<Contribution> = (0..amounts.len())
        .map(|i| Contribution::new(VAULT, funder(i)))
        .collect();
    for (c, &a) in cs.iter_mut().zip(amounts) {
        v.contribute(c, Usdc::new(a), 0).unwrap();
    }
    (v, cs)
}

proptest! {
    #[test]
    fn refunds_sum_to_total_contributed(
        (amounts, order) in prop::collection::vec(MIN_CONTRIBUTION.micro()..=1_000_000_000_000, 1..20)
            .prop_flat_map(|a| {
                let n = a.len();
                (Just(a), Just((0..n).collect::<Vec<_>>()).prop_shuffle())
            }),
    ) {
        let (mut v, mut cs) = vault_with(&amounts);
        v.expire(100).unwrap();

        // Refund via Vault::refund in a random permutation; every funder paid exactly once.
        let mut paid = Usdc::ZERO;
        for i in order {
            v.refund(&mut cs[i]).unwrap();
            paid = paid.checked_add(cs[i].amount()).unwrap();
            prop_assert_eq!(v.refund(&mut cs[i]), Err(DomainError::AlreadyRefunded));
        }
        prop_assert_eq!(paid, v.total_contributed());
        prop_assert_eq!(v.refunded_total(), v.total_contributed());
        prop_assert_eq!(v.status(), VaultStatus::Closed);
    }

    #[test]
    fn total_equals_sum_of_contributions(
        ops in prop::collection::vec((0usize..5, MIN_CONTRIBUTION.micro()..=1_000_000_000), 1..40),
    ) {
        let (mut v, mut cs) = vault_with(&[]);
        cs.extend((0..5).map(|i| Contribution::new(VAULT, funder(i))));
        for (i, a) in ops {
            v.contribute(&mut cs[i], Usdc::new(a), 0).unwrap();
        }
        let sum: u64 = cs.iter().map(|c| c.amount().micro()).sum();
        let funded = cs.iter().filter(|c| !c.amount().is_zero()).count() as u32;
        prop_assert_eq!(v.total_contributed().micro(), sum);
        prop_assert_eq!(v.contributor_count(), funded);
    }
}
