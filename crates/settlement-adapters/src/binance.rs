//! Adaptador stub para Binance CEX (API simulada + spread buffer).
//!
//! Implementación completa: paso 4.3 del plan.

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

/// Liquidación vía débito simulado en exchange custodial.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BinanceCexAdapter;

#[async_trait]
impl SettlementAdapter for BinanceCexAdapter {
    fn rail(&self) -> FundingType {
        FundingType::BinanceCex
    }

    async fn settle(&self, _context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        Err(LiquidityError::SettlementFailed)
    }
}
