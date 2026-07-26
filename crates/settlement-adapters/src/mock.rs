//! Adaptador mock para tests de integración del Settlement Engine.

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

/// Adaptador configurable que devuelve una prueba fija (tests y mocks del Gateway).
#[derive(Debug, PartialEq, Eq)]
pub struct MockSettlementAdapter {
    rail: FundingType,
    proof: String,
    fail_with: Option<LiquidityError>,
}

impl MockSettlementAdapter {
    /// Crea un mock que responde con la prueba indicada.
    pub fn new(rail: FundingType, proof: impl Into<String>) -> Self {
        Self {
            rail,
            proof: proof.into(),
            fail_with: None,
        }
    }

    /// Configura el mock para fallar con el error dado.
    pub fn with_error(mut self, error: LiquidityError) -> Self {
        self.fail_with = Some(error);
        self
    }
}

#[async_trait]
impl SettlementAdapter for MockSettlementAdapter {
    fn rail(&self) -> FundingType {
        self.rail
    }

    async fn settle(&self, _context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }

        Ok(SettlementReceipt {
            proof: self.proof.clone(),
            rail: self.rail,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};
    use uuid::Uuid;

    use super::*;

    fn sample_context() -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::new_v4()),
            transaction_id: TransactionId::generate(),
            merchant_id: MerchantId::new(Uuid::new_v4()),
            amount: Amount::from_units(50),
            currency: Currency::from_str("USD").expect("currency"),
            brand_code: 2,
            settlement_rail_id: 3,
        }
    }

    #[tokio::test]
    async fn returns_configured_proof() {
        let adapter = MockSettlementAdapter::new(FundingType::TraditionalBank, "bank-ref-99");
        let receipt = adapter.settle(sample_context()).await.expect("receipt");

        assert_eq!(receipt.proof, "bank-ref-99");
        assert_eq!(receipt.rail, FundingType::TraditionalBank);
    }

    #[tokio::test]
    async fn returns_configured_error() {
        let adapter = MockSettlementAdapter::new(FundingType::SolanaWallet, "unused").with_error(
            LiquidityError::HoldNotFound,
        );

        assert_eq!(
            adapter.settle(sample_context()).await,
            Err(LiquidityError::HoldNotFound)
        );
    }
}
