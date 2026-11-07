//! Maps RPC and program failures to `PortError`, program codes back to their domain rule.

use anchor_client::compat::solana_rpc_client_api::request::RpcError;
use anchor_client::compat::solana_transaction::{InstructionError, TransactionError};
use anchor_client::{ClientError, SolanaClientError, SolanaClientErrorKind};
use arrhabon::error::to_anchor;
use arrhabon_app::PortError;
use arrhabon_domain::DomainError;

/// Every domain rule; the program's `to_anchor` is the single code mapping.
const ALL: [DomainError; 22] = {
    use DomainError::*;
    [
        ZeroAmount,
        Overflow,
        Underflow,
        DeadlinePassed,
        DeadlineNotReached,
        NotOpen,
        NotRefunding,
        NotReleased,
        BelowTarget,
        Unauthorized,
        AlreadyRefunded,
        NothingToRefund,
        ContributionMismatch,
        WrongVault,
        InvalidParties,
        CorruptState,
        DeadlineTooFar,
        BelowMinimum,
        ExceedsTarget,
        NotClosed,
        BalanceRemaining,
        ContributionsOpen,
    ]
};

/// The domain rule behind a program custom error code; `None` for non-domain codes.
pub fn domain_code(code: u32) -> Option<DomainError> {
    ALL.into_iter().find(|d| anchor_code(*d) == Some(code))
}

fn anchor_code(d: DomainError) -> Option<u32> {
    match to_anchor(d) {
        anchor_lang::error::Error::AnchorError(e) => Some(e.error_code_number),
        anchor_lang::error::Error::ProgramError(_) => None,
    }
}

/// Message for a send whose blockhash expired before the tx reached confirmed commitment.
pub const EXPIRED_UNCONFIRMED: &str = "transaction blockhash expired before confirmation; \
     it may still have landed: re-read on-chain state before retrying";

/// `send_and_confirm_transaction` gives up with this `ForUser` message once the blockhash
/// is no longer valid and no status was seen.
fn expired_unconfirmed(e: &SolanaClientError) -> bool {
    matches!(e.kind(), SolanaClientErrorKind::RpcError(RpcError::ForUser(m))
        if m.starts_with("unable to confirm transaction"))
}

/// A failed send, simulation or confirmation. Transport errors lose their URL, which may
/// carry an RPC API key.
pub fn from_rpc(e: SolanaClientError) -> PortError {
    if expired_unconfirmed(&e) {
        return PortError::Unavailable(EXPIRED_UNCONFIRMED.into());
    }
    match e.get_transaction_error() {
        Some(TransactionError::AlreadyProcessed) => PortError::AlreadyApplied,
        Some(TransactionError::InstructionError(_, InstructionError::Custom(code))) => {
            PortError::Rejected {
                code: domain_code(code),
                message: e.to_string(),
            }
        }
        Some(other) => PortError::rejected(other.to_string()),
        None => PortError::Unavailable(match *e.kind {
            SolanaClientErrorKind::Reqwest(r) => r.without_url().to_string(),
            other => other.to_string(),
        }),
    }
}

pub fn from_client(e: ClientError) -> PortError {
    match e {
        ClientError::SolanaClientError(e) => from_rpc(*e),
        other => PortError::rejected(other.to_string()),
    }
}

/// Crank calls: a rejection with one of `codes` means the effect is already on chain.
pub fn applied_if(e: PortError, codes: &[DomainError]) -> PortError {
    match e {
        PortError::Rejected { code: Some(c), .. } if codes.contains(&c) => {
            PortError::AlreadyApplied
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_client::SolanaClientErrorKind;

    fn custom(code: u32) -> SolanaClientError {
        SolanaClientErrorKind::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(code),
        ))
        .into()
    }

    #[test]
    fn every_domain_code_round_trips_through_the_program() {
        for d in ALL {
            let code = anchor_code(d).unwrap();
            assert_eq!(domain_code(code), Some(d), "{d:?} -> {code}");
        }
        assert_eq!(domain_code(6000), Some(DomainError::ZeroAmount));
    }

    #[test]
    fn non_domain_program_codes_carry_no_domain_rule() {
        let mint_mismatch = u32::from(arrhabon::error::ArrhabonError::MintMismatch);
        assert_eq!(domain_code(mint_mismatch), None);
        assert_eq!(domain_code(0), None);
    }

    #[test]
    fn custom_error_maps_to_rejected_with_domain_code() {
        let not_open = anchor_code(DomainError::NotOpen).unwrap();
        match from_rpc(custom(not_open)) {
            PortError::Rejected { code, .. } => assert_eq!(code, Some(DomainError::NotOpen)),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn already_processed_is_already_applied() {
        let e: SolanaClientError =
            SolanaClientErrorKind::TransactionError(TransactionError::AlreadyProcessed).into();
        assert_eq!(from_rpc(e), PortError::AlreadyApplied);
    }

    #[test]
    fn transport_failure_is_unavailable() {
        let e: SolanaClientError =
            SolanaClientErrorKind::Custom("connection refused".into()).into();
        assert!(matches!(from_rpc(e), PortError::Unavailable(_)));
    }

    #[tokio::test]
    async fn transport_errors_never_carry_the_rpc_url() {
        // Port 1 on loopback refuses at once; nothing leaves the machine.
        let url = "http://127.0.0.1:1/?api-key=SECRET";
        let rpc = anchor_client::compat::solana_rpc_client::nonblocking::rpc_client::RpcClient::new(
            url.into(),
        );
        let e = rpc.get_genesis_hash().await.unwrap_err();
        assert!(
            e.to_string().contains("SECRET"),
            "precondition: raw error names the URL"
        );
        match from_rpc(e) {
            PortError::Unavailable(m) => {
                assert!(!m.contains("SECRET") && !m.contains("127.0.0.1"), "{m}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn expired_blockhash_tells_callers_to_re_read_state() {
        let e: SolanaClientError = SolanaClientErrorKind::RpcError(RpcError::ForUser(
            "unable to confirm transaction. This can happen in situations such as transaction \
             expiration and insufficient fee-payer funds"
                .into(),
        ))
        .into();
        assert_eq!(
            from_rpc(e),
            PortError::Unavailable(EXPIRED_UNCONFIRMED.into())
        );
        assert!(EXPIRED_UNCONFIRMED.contains("re-read"));
    }

    #[test]
    fn crank_rejections_already_on_chain_become_already_applied() {
        let not_open = from_rpc(custom(anchor_code(DomainError::NotOpen).unwrap()));
        assert_eq!(
            applied_if(not_open, &[DomainError::NotOpen]),
            PortError::AlreadyApplied
        );
        let below = from_rpc(custom(anchor_code(DomainError::BelowTarget).unwrap()));
        assert!(matches!(
            applied_if(below, &[DomainError::NotOpen]),
            PortError::Rejected { .. }
        ));
    }
}
