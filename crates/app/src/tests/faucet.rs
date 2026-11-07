use arrhabon_domain::{AccountId, Usdc};

use super::run;
use crate::AppError;
use crate::commands::{ClaimTestFunds, LAMPORTS_GRANT, USDC_GRANT};
use crate::fakes::FakeFaucet;

const JUDGE: AccountId = AccountId([9; 32]);

#[test]
fn an_empty_wallet_gets_the_full_grant() {
    let faucet = FakeFaucet::default();
    let grant = run(ClaimTestFunds { wallet: JUDGE }.execute(&faucet)).unwrap();
    assert_eq!((grant.lamports, grant.usdc), (LAMPORTS_GRANT, USDC_GRANT));
    assert_eq!(faucet.sent(), vec![(JUDGE, LAMPORTS_GRANT, USDC_GRANT)]);
}

#[test]
fn only_the_shortfall_is_sent() {
    let faucet = FakeFaucet::default();
    faucet.set(JUDGE, LAMPORTS_GRANT, Usdc::new(400_000_000));
    let grant = run(ClaimTestFunds { wallet: JUDGE }.execute(&faucet)).unwrap();
    assert_eq!((grant.lamports, grant.usdc), (0, Usdc::new(600_000_000)));
}

#[test]
fn a_funded_wallet_gets_nothing() {
    let faucet = FakeFaucet::default();
    faucet.set(JUDGE, LAMPORTS_GRANT + 1, Usdc::new(USDC_GRANT.micro() + 1));
    let err = run(ClaimTestFunds { wallet: JUDGE }.execute(&faucet)).unwrap_err();
    assert_eq!(err, AppError::AlreadyFunded);
    assert!(faucet.sent().is_empty());
}

#[test]
fn a_second_claim_right_after_the_first_is_refused() {
    let faucet = FakeFaucet::default();
    run(ClaimTestFunds { wallet: JUDGE }.execute(&faucet)).unwrap();
    let err = run(ClaimTestFunds { wallet: JUDGE }.execute(&faucet)).unwrap_err();
    assert_eq!(err, AppError::AlreadyFunded);
    assert_eq!(faucet.sent().len(), 1);
}
