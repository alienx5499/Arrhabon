//! Cross-chain funding quotes, checked against the vault before and after quoting.

use arrhabon_domain::AccountId;

use super::{check_funder, load_vault, precheck_contribution};
use crate::AppError;
use crate::ports::{Clock, DestinationAsset, QuoteRequest, SwapQuote, SwapQuoter, VaultLedger};

const MAX_SLIPPAGE_BPS: u16 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteCrossChainFunding {
    pub vault: AccountId,
    pub origin_chain: String,
    pub origin_asset: String,
    pub origin_amount: u128,
    /// The funder's Solana wallet: receives the USDC, then contributes.
    pub funder: AccountId,
    pub refund_to: String,
    pub slippage_bps: u16,
}

impl QuoteCrossChainFunding {
    /// Returns a quote whose expected USDC would be a valid contribution right now.
    pub async fn execute(
        self,
        ledger: &impl VaultLedger,
        quoter: &impl SwapQuoter,
        clock: &impl Clock,
    ) -> Result<SwapQuote, AppError> {
        if self.origin_amount == 0 {
            return Err(AppError::InvalidInput("origin amount must be positive"));
        }
        if self.slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(AppError::InvalidInput("slippage must be at most 10000 bps"));
        }
        if [&self.origin_chain, &self.origin_asset, &self.refund_to]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            return Err(AppError::InvalidInput(
                "origin chain, asset and refund address are required",
            ));
        }
        check_funder(self.funder)?;
        let vault = load_vault(ledger, self.vault).await?;
        vault.accepts_contributions(clock.now())?;
        let quote = quoter
            .quote(QuoteRequest {
                origin_chain: self.origin_chain,
                origin_asset: self.origin_asset,
                origin_amount: self.origin_amount,
                recipient: self.funder,
                refund_to: self.refund_to,
                slippage_bps: self.slippage_bps,
                destination: DestinationAsset::SolanaUsdc,
            })
            .await?;
        precheck_contribution(ledger, &vault, self.funder, quote.min_usdc_out, clock.now()).await?;
        Ok(quote)
    }
}
