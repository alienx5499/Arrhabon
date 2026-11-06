#![allow(dead_code)]

use arrhabon_domain::{AccountId, Contribution, Deadline, Usdc, Vault};

pub const VAULT: AccountId = AccountId([7; 32]);
pub const OTHER_VAULT: AccountId = AccountId([8; 32]);
pub const STUDENT: AccountId = AccountId([1; 32]);
pub const UNIVERSITY: AccountId = AccountId([2; 32]);
pub const ATTESTOR: AccountId = AccountId([3; 32]);
pub const MOM: AccountId = AccountId([4; 32]);
pub const UNCLE: AccountId = AccountId([5; 32]);
pub const STRANGER: AccountId = AccountId([9; 32]);
pub const NOW: i64 = 1_000;
pub const DEADLINE: i64 = 2_000;

/// `n` whole USDC (contributions must be at least `MIN_CONTRIBUTION`, 1 USDC).
pub fn usdc(n: u64) -> Usdc {
    Usdc::new(n * 1_000_000)
}

pub fn open_vault() -> Vault {
    Vault::create(
        VAULT,
        STUDENT,
        UNIVERSITY,
        ATTESTOR,
        usdc(1_000),
        Deadline::new(DEADLINE),
        NOW,
    )
    .unwrap()
}

/// Vault with MOM 600 and UNCLE 400 contributed.
pub fn funded() -> (Vault, Contribution, Contribution) {
    let mut v = open_vault();
    let mut mom = Contribution::new(VAULT, MOM);
    let mut uncle = Contribution::new(VAULT, UNCLE);
    v.contribute(&mut mom, usdc(600), NOW).unwrap();
    v.contribute(&mut uncle, usdc(400), NOW).unwrap();
    (v, mom, uncle)
}
