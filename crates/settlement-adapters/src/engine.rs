//! Motor de liquidación — despacha al adaptador del riel activo.

use std::collections::HashMap;
use std::sync::Arc;

use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::bank::TraditionalBankAdapter;
use crate::binance::BinanceCexAdapter;
use crate::context::SettlementContext;
use crate::solana::SolanaWalletAdapter;

/// Orquestador de adaptadores de liquidación (Settlement Engine).
///
/// Resuelve el adaptador correspondiente al [`FundingType`] seleccionado
/// por el Rail Switcher e invoca `settle`.
#[derive(Clone)]
pub struct SettlementEngine {
    adapters: HashMap<FundingType, Arc<dyn SettlementAdapter>>,
}

impl SettlementEngine {
    /// Construye el motor a partir de una lista de adaptadores registrados.
    ///
    /// # Inputs
    /// - `adapters`: implementaciones concretas por riel.
    ///
    /// # Returns
    /// Motor listo para despachar liquidaciones.
    pub fn new(adapters: impl IntoIterator<Item = Arc<dyn SettlementAdapter>>) -> Self {
        let mut map = HashMap::new();
        for adapter in adapters {
            map.insert(adapter.rail(), adapter);
        }
        Self { adapters: map }
    }

    /// Crea un motor con los tres stubs por defecto (pasos 4.2–4.4 los reemplazan).
    pub fn with_stub_adapters() -> Self {
        Self::new([
            Arc::new(TraditionalBankAdapter) as Arc<dyn SettlementAdapter>,
            Arc::new(BinanceCexAdapter),
            Arc::new(SolanaWalletAdapter),
        ])
    }

    /// Indica si hay un adaptador registrado para el riel dado.
    pub fn supports(&self, rail: FundingType) -> bool {
        self.adapters.contains_key(&rail)
    }

    /// Devuelve el adaptador del riel o error si no está registrado.
    pub fn adapter_for(&self, rail: FundingType) -> Result<&dyn SettlementAdapter, LiquidityError> {
        self.adapters
            .get(&rail)
            .map(|adapter| adapter.as_ref())
            .ok_or(LiquidityError::RailUnavailable { rail })
    }

    /// Ejecuta la liquidación en el riel indicado.
    ///
    /// # Inputs
    /// - `rail`: riel seleccionado por el Rail Switcher.
    /// - `context`: datos de la transacción y hold a consumir.
    ///
    /// # Returns
    /// Comprobante de asentamiento o error de liquidez.
    pub async fn settle(
        &self,
        rail: FundingType,
        context: SettlementContext,
    ) -> Result<SettlementReceipt, LiquidityError> {
        let adapter = self.adapter_for(rail)?;
        adapter.settle(context).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};
    use uuid::Uuid;

    use super::*;
    use crate::mock::MockSettlementAdapter;

    fn sample_context() -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::new_v4()),
            transaction_id: TransactionId::generate(),
            merchant_id: MerchantId::new(Uuid::new_v4()),
            amount: Amount::from_units(100),
            currency: Currency::from_str("USD").expect("currency"),
            brand_code: 1,
            settlement_rail_id: 1,
        }
    }

    #[test]
    fn with_stub_adapters_registers_all_rails() {
        let engine = SettlementEngine::with_stub_adapters();
        assert!(engine.supports(FundingType::TraditionalBank));
        assert!(engine.supports(FundingType::BinanceCex));
        assert!(engine.supports(FundingType::SolanaWallet));
    }

    #[test]
    fn adapter_for_unknown_rail_returns_unavailable() {
        let engine = SettlementEngine::new([]);
        let result = engine.adapter_for(FundingType::SolanaWallet);
        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some(LiquidityError::RailUnavailable {
                rail: FundingType::SolanaWallet
            })
        );
    }

    #[tokio::test]
    async fn settle_dispatches_to_matching_adapter() {
        let mock = Arc::new(MockSettlementAdapter::new(
            FundingType::BinanceCex,
            "cex-order-42",
        )) as Arc<dyn SettlementAdapter>;
        let engine = SettlementEngine::new([mock]);
        let context = sample_context();

        let receipt = engine
            .settle(FundingType::BinanceCex, context)
            .await
            .expect("settled");

        assert_eq!(receipt.proof, "cex-order-42");
        assert_eq!(receipt.rail, FundingType::BinanceCex);
    }

    #[tokio::test]
    async fn settle_returns_unavailable_when_rail_not_registered() {
        let engine = SettlementEngine::new([]);
        let result = engine
            .settle(FundingType::TraditionalBank, sample_context())
            .await;

        assert_eq!(
            result,
            Err(LiquidityError::RailUnavailable {
                rail: FundingType::TraditionalBank
            })
        );
    }

    #[tokio::test]
    async fn stub_adapters_return_settlement_failed_until_implemented() {
        let engine = SettlementEngine::with_stub_adapters();
        let context = sample_context();

        for rail in [
            FundingType::TraditionalBank,
            FundingType::BinanceCex,
            FundingType::SolanaWallet,
        ] {
            assert_eq!(
                engine.settle(rail, context.clone()).await,
                Err(LiquidityError::SettlementFailed)
            );
        }
    }
}
