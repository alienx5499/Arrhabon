//! DEMO ONLY: tops a wallet up to the grant, so a judge can create and fund a vault.

use arrhabon_domain::{AccountId, Usdc};

use crate::AppError;
use crate::ports::{TestFunds, TxSignature};

/// 0.05 SOL: enough fees for dozens of vault transactions.
pub const LAMPORTS_GRANT: u64 = 50_000_000;
/// 1,000 test USDC.
pub const USDC_GRANT: Usdc = Usdc::new(1_000_000_000);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub signature: TxSignature,
    pub lamports: u64,
    pub usdc: Usdc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimTestFunds {
    pub wallet: AccountId,
}

impl ClaimTestFunds {
    /// Sends only the shortfall up to each grant, so repeat claims cannot farm the faucet.
    pub async fn execute(self, faucet: &impl TestFunds) -> Result<Grant, AppError> {
        let has = faucet.funds(self.wallet).await?;
        let lamports = LAMPORTS_GRANT.saturating_sub(has.lamports);
        let usdc = Usdc::new(USDC_GRANT.micro().saturating_sub(has.usdc.micro()));
        if lamports == 0 && usdc.is_zero() {
            return Err(AppError::AlreadyFunded);
        }
        let signature = faucet.send(self.wallet, lamports, usdc).await?;
        Ok(Grant {
            signature,
            lamports,
            usdc,
        })
    }
}
