//! Adaptador de liquidación — riel bancario tradicional (ISO 20022 / ACH simulado).

mod compensation;

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

pub use compensation::{BankCompensation, Pacs008Document, generate as generate_bank_compensation};

/// Liquidación vía compensación bancaria simulada (pacs.008 / ACH).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TraditionalBankAdapter;

impl TraditionalBankAdapter {
    /// Genera la compensación completa (referencia + mensaje ISO 20022) sin persistir.
    ///
    /// # Inputs
    /// - `context`: datos de la transacción con hold activo.
    ///
    /// # Returns
    /// [`BankCompensation`] listo para auditoría y prueba de asentamiento.
    pub fn compensate(context: &SettlementContext) -> Result<BankCompensation, LiquidityError> {
        compensation::generate(context)
    }
}

#[async_trait]
impl SettlementAdapter for TraditionalBankAdapter {
    fn rail(&self) -> FundingType {
        FundingType::TraditionalBank
    }

    async fn settle(&self, context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        let compensation = compensation::generate(&context)?;

        Ok(SettlementReceipt {
            proof: compensation.reference_id,
            rail: FundingType::TraditionalBank,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};
    use uuid::Uuid;

    use super::*;
    use crate::SettlementEngine;

    fn sample_context() -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::new_v4()),
            transaction_id: TransactionId::generate(),
            merchant_id: MerchantId::new(Uuid::new_v4()),
            amount: Amount::from_units(150),
            currency: Currency::from_str("EUR").expect("currency"),
            brand_code: 2,
            settlement_rail_id: 1,
        }
    }

    #[tokio::test]
    async fn settle_returns_bank_reference_as_proof() {
        let adapter = TraditionalBankAdapter;
        let context = sample_context();
        let expected = compensation::generate(&context)
            .expect("compensation")
            .reference_id;

        let receipt = adapter.settle(context).await.expect("settled");

        assert_eq!(receipt.proof, expected);
        assert_eq!(receipt.rail, FundingType::TraditionalBank);
        assert!(receipt.proof.starts_with("ACH-"));
    }

    #[tokio::test]
    async fn engine_settles_via_traditional_bank_adapter() {
        let engine = SettlementEngine::new([Arc::new(TraditionalBankAdapter) as Arc<dyn SettlementAdapter>]);
        let context = sample_context();

        let receipt = engine
            .settle(FundingType::TraditionalBank, context)
            .await
            .expect("settled");

        assert_eq!(receipt.rail, FundingType::TraditionalBank);
        assert!(receipt.proof.starts_with("ACH-"));
    }
}
