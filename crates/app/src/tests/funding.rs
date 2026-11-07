//! Cross-chain funding quotes: validated against the vault before a funder deposits.

use arrhabon_domain::DomainError;

use super::*;
use crate::AppError;
use crate::commands::{MAX_MIRROR_USDC, MirrorSettledSwap, QuoteCrossChainFunding, SwapRecord};
use crate::fakes::{FakeFaucet, FakeQuoter};
use crate::ports::{SwapQuoter, SwapStatus};

fn quote_cmd(vault: AccountId, amount: u128) -> QuoteCrossChainFunding {
    QuoteCrossChainFunding {
        vault,
        origin_chain: "base".into(),
        origin_asset: "usdc".into(),
        origin_amount: amount,
        funder: UNCLE,
        refund_to: "0xuncle".into(),
        slippage_bps: 50,
    }
}

#[test]
fn quote_cross_chain_funding_returns_a_usable_quote() {
    let (clock, ledger) = setup();
    let quoter = FakeQuoter::new(clock.clone(), Usdc::new(200_000));
    let vault = seed_vault(&ledger, 1, NOW + 120, &[]);
    let quote = run(quote_cmd(vault, 50_000_000).execute(&ledger, &quoter, &clock)).unwrap();
    assert_eq!(quote.min_usdc_out, Usdc::new(49_800_000));
    assert_eq!(quote.expires_at, NOW + 600);
    assert_eq!(
        run(quoter.status(&quote.deposit_address)),
        Ok(SwapStatus::Pending)
    );
    quoter.set_status(&quote.deposit_address, SwapStatus::Refunded);
    assert_eq!(
        run(quoter.status(&quote.deposit_address)),
        Ok(SwapStatus::Refunded)
    );
}

#[test]
fn quote_cross_chain_funding_failures() {
    let (clock, ledger) = setup();
    let quoter = FakeQuoter::new(clock.clone(), Usdc::new(200_000));
    let vault = seed_vault(&ledger, 1, NOW + 120, &[]);
    let quote = |cmd: QuoteCrossChainFunding| run(cmd.execute(&ledger, &quoter, &clock));

    assert!(matches!(
        quote(quote_cmd(vault, 0)),
        Err(AppError::InvalidInput(_))
    ));
    let mut blank = quote_cmd(vault, 5_000_000);
    blank.refund_to = " ".into();
    assert!(matches!(quote(blank), Err(AppError::InvalidInput(_))));
    let mut greedy = quote_cmd(vault, 5_000_000);
    greedy.slippage_bps = 10_001;
    assert!(matches!(quote(greedy), Err(AppError::InvalidInput(_))));
    // 1 USDC in, 0.8 out after the fee: below the domain's minimum contribution.
    assert_eq!(
        quote(quote_cmd(vault, 1_000_000)),
        Err(AppError::Invalid(DomainError::BelowMinimum))
    );
    assert!(matches!(
        quote(quote_cmd(vault, 100)),
        Err(AppError::Rejected(_))
    ));
}

#[test]
fn quote_cross_chain_funding_checks_the_vault_before_asking_for_a_quote() {
    let (clock, ledger) = setup();
    let quoter = FakeQuoter::new(clock.clone(), Usdc::new(200_000));
    let open = seed_vault(&ledger, 1, NOW + 120, &[(AUNT, 100)]);
    let missing = AccountId([9; 32]);
    let quote = |vault| run(quote_cmd(vault, 5_000_000).execute(&ledger, &quoter, &clock));

    assert_eq!(quote(missing), Err(AppError::VaultNotFound(missing)));
    clock.set(NOW + 120);
    assert_eq!(
        quote(open),
        Err(AppError::Conflict(DomainError::DeadlinePassed))
    );
    // No deposit address was ever handed out.
    assert!(run(quoter.status("deposit-1")).is_err());
}

#[test]
fn mirrors_success_once() {
    let (clock, _) = setup();
    let quoter = FakeQuoter::new(clock, Usdc::new(200_000));
    quoter.set_status("deposit-1", SwapStatus::Success);
    let faucet = FakeFaucet::default();
    let cmd = MirrorSettledSwap {
        deposit_address: "deposit-1".into(),
        record: SwapRecord {
            funder: UNCLE,
            vault: AccountId([10; 32]),
            amount: Usdc::new(50_000_000),
        },
    };
    let mirror = run(cmd.execute(&quoter, &faucet)).unwrap();
    assert_eq!(mirror.usdc, Usdc::new(50_000_000));
    assert_eq!(mirror.funder, UNCLE);
    assert_eq!(mirror.vault, AccountId([10; 32]));
    assert_eq!(faucet.sent(), vec![(UNCLE, 0, Usdc::new(50_000_000))]);
}

#[test]
fn refuses_pending() {
    let (clock, _) = setup();
    let quoter = FakeQuoter::new(clock, Usdc::new(200_000));
    quoter.set_status("deposit-1", SwapStatus::Pending);
    let faucet = FakeFaucet::default();
    let cmd = MirrorSettledSwap {
        deposit_address: "deposit-1".into(),
        record: SwapRecord {
            funder: UNCLE,
            vault: AccountId([10; 32]),
            amount: Usdc::new(50_000_000),
        },
    };
    assert_eq!(
        run(cmd.execute(&quoter, &faucet)),
        Err(AppError::SwapPending)
    );
    assert!(faucet.sent().is_empty());
}

#[test]
fn refuses_refunded_and_failed() {
    let (clock, _) = setup();
    let quoter = FakeQuoter::new(clock, Usdc::new(200_000));
    let faucet = FakeFaucet::default();
    let cmd = MirrorSettledSwap {
        deposit_address: "deposit-1".into(),
        record: SwapRecord {
            funder: UNCLE,
            vault: AccountId([10; 32]),
            amount: Usdc::new(50_000_000),
        },
    };
    quoter.set_status("deposit-1", SwapStatus::Refunded);
    assert_eq!(
        run(cmd.execute(&quoter, &faucet)),
        Err(AppError::SwapNotSettled)
    );
    quoter.set_status("deposit-1", SwapStatus::Failed);
    assert_eq!(
        run(cmd.execute(&quoter, &faucet)),
        Err(AppError::SwapNotSettled)
    );
    assert!(faucet.sent().is_empty());
}

#[test]
fn caps_amount_at_100_usdc() {
    let (clock, _) = setup();
    let quoter = FakeQuoter::new(clock, Usdc::new(200_000));
    quoter.set_status("deposit-1", SwapStatus::Success);
    let faucet = FakeFaucet::default();
    let cmd = MirrorSettledSwap {
        deposit_address: "deposit-1".into(),
        record: SwapRecord {
            funder: UNCLE,
            vault: AccountId([10; 32]),
            amount: Usdc::new(250_000_000),
        },
    };
    let mirror = run(cmd.execute(&quoter, &faucet)).unwrap();
    assert_eq!(mirror.usdc, MAX_MIRROR_USDC);
    assert_eq!(faucet.sent(), vec![(UNCLE, 0, MAX_MIRROR_USDC)]);
}

#[test]
fn releases_claim_when_send_fails() {
    let (clock, _) = setup();
    let quoter = FakeQuoter::new(clock, Usdc::new(200_000));
    quoter.set_status("deposit-1", SwapStatus::Success);
    let faucet = FakeFaucet::default();
    faucet.set_failing(true);
    let cmd = MirrorSettledSwap {
        deposit_address: "deposit-1".into(),
        record: SwapRecord {
            funder: UNCLE,
            vault: AccountId([10; 32]),
            amount: Usdc::new(50_000_000),
        },
    };
    let err = run(cmd.execute(&quoter, &faucet)).unwrap_err();
    assert!(matches!(err, AppError::Unavailable(_)));
    assert!(faucet.sent().is_empty());
}
