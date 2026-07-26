//! Adaptador stub para riel bancario tradicional (ISO 20022 / ACH simulado).
//!
//! Implementación completa: paso 4.2 del plan.

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

/// Liquidación vía compensación bancaria simulada.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TraditionalBankAdapter;

#[async_trait]
impl SettlementAdapter for TraditionalBankAdapter {
    fn rail(&self) -> FundingType {
        FundingType::TraditionalBank
    }

    async fn settle(&self, _context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        Err(LiquidityError::SettlementFailed)
    }
}
