//! Adaptador stub para Solana on-chain (`process_payment` + commitment finalized).
//!
//! Implementación completa: paso 4.4 del plan.

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

/// Liquidación vía programa Anchor `payment-settlement`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SolanaWalletAdapter;

#[async_trait]
impl SettlementAdapter for SolanaWalletAdapter {
    fn rail(&self) -> FundingType {
        FundingType::SolanaWallet
    }

    async fn settle(&self, _context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        Err(LiquidityError::SettlementFailed)
    }
}
