//! Adaptador de liquidación — riel Binance CEX (API simulada + spread buffer).

mod client;
mod config;

use std::sync::Arc;

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};
use rust_decimal::Decimal;

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

pub use client::{
    BinanceClientError, BinanceSpotClient, HttpBinanceSpotClient, InMemoryBinanceSpotClient,
    SpotDebitRequest, SpotDebitResponse,
};
pub use config::{BinanceCexConfig, BinanceConfigError};

/// Liquidación vía débito Spot custodial simulado (USDC/USDT).
#[derive(Clone)]
pub struct BinanceCexAdapter {
    client: Arc<dyn BinanceSpotClient>,
    spread_buffer_pct: Decimal,
}

impl BinanceCexAdapter {
    /// Crea el adaptador con cliente HTTP y spread buffer configurables.
    pub fn new(client: Arc<dyn BinanceSpotClient>, spread_buffer_pct: Decimal) -> Self {
        Self {
            client,
            spread_buffer_pct,
        }
    }

    /// Adaptador con cliente HTTP cargado desde entorno.
    pub fn from_env() -> Result<Self, BinanceConfigError> {
        let config = BinanceCexConfig::from_env()?;
        let spread = config.spread_buffer_pct;
        let client = HttpBinanceSpotClient::new(&config)
            .map_err(|_| BinanceConfigError::ClientInitFailed)?;
        Ok(Self::new(Arc::new(client), spread))
    }

    /// Adaptador en memoria para tests (saldo inicial 5000, spread 2%).
    pub fn in_memory_mock() -> Self {
        Self::new(
            Arc::new(InMemoryBinanceSpotClient::with_balance(Decimal::from(5000))),
            Decimal::new(2, 2),
        )
    }

    /// Ejecuta débito Spot y retorna la respuesta completa de la API.
    pub async fn debit(
        &self,
        context: &SettlementContext,
    ) -> Result<SpotDebitResponse, LiquidityError> {
        validate_amount(context)?;
        let currency = normalize_binance_currency(context.currency.as_str())
            .map_err(map_client_error)?;
        let request = SpotDebitRequest {
            amount: context.amount.value(),
            currency,
            client_order_id: context.hold_id.0.to_string(),
            spread_buffer_pct: self.spread_buffer_pct,
        };

        self.client
            .debit(request)
            .await
            .map_err(map_client_error)
    }
}

impl Default for BinanceCexAdapter {
    fn default() -> Self {
        Self::in_memory_mock()
    }
}

#[async_trait]
impl SettlementAdapter for BinanceCexAdapter {
    fn rail(&self) -> FundingType {
        FundingType::BinanceCex
    }

    async fn settle(&self, context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        let response = self.debit(&context).await?;

        Ok(SettlementReceipt {
            proof: response.order_id,
            rail: FundingType::BinanceCex,
        })
    }
}

fn validate_amount(context: &SettlementContext) -> Result<(), LiquidityError> {
    if context.amount.is_positive() {
        Ok(())
    } else {
        Err(LiquidityError::InsufficientFunds {
            rail: FundingType::BinanceCex,
        })
    }
}

/// Mapea moneda de checkout a par Spot soportado (USD → USDC).
pub fn normalize_binance_currency(currency: &str) -> Result<String, BinanceClientError> {
    let normalized = currency.to_ascii_uppercase();
    match normalized.as_str() {
        "USD" => Ok("USDC".to_string()),
        "USDC" | "USDT" => Ok(normalized),
        _ => Err(BinanceClientError::UnsupportedCurrency),
    }
}

fn map_client_error(error: BinanceClientError) -> LiquidityError {
    match error {
        BinanceClientError::InsufficientFunds => LiquidityError::InsufficientFunds {
            rail: FundingType::BinanceCex,
        },
        BinanceClientError::UnsupportedCurrency => LiquidityError::SettlementFailed,
        BinanceClientError::InvalidRequest => LiquidityError::SettlementFailed,
        BinanceClientError::Unavailable(_) => LiquidityError::RailUnavailable {
            rail: FundingType::BinanceCex,
        },
        BinanceClientError::InvalidResponse(_) => LiquidityError::SettlementFailed,
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

    fn sample_context(amount_units: i64) -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::new_v4()),
            transaction_id: TransactionId::generate(),
            merchant_id: MerchantId::new(Uuid::new_v4()),
            amount: Amount::from_units(amount_units),
            currency: Currency::from_str("USD").expect("currency"),
            brand_code: 1,
            settlement_rail_id: 2,
        }
    }

    #[test]
    fn normalize_maps_usd_to_usdc() {
        assert_eq!(
            normalize_binance_currency("usd").expect("ok"),
            "USDC"
        );
    }

    #[tokio::test]
    async fn settle_returns_cex_order_id_as_proof() {
        let adapter = BinanceCexAdapter::in_memory_mock();
        let receipt = adapter
            .settle(sample_context(100))
            .await
            .expect("settled");

        assert_eq!(receipt.rail, FundingType::BinanceCex);
        assert!(receipt.proof.starts_with("CEX-MEM-"));
    }

    #[tokio::test]
    async fn rejects_non_positive_amount() {
        let adapter = BinanceCexAdapter::in_memory_mock();
        let result = adapter.settle(sample_context(0)).await;

        assert_eq!(
            result,
            Err(LiquidityError::InsufficientFunds {
                rail: FundingType::BinanceCex
            })
        );
    }

    #[tokio::test]
    async fn rejects_when_spread_buffer_blocks_debit() {
        let adapter = BinanceCexAdapter::new(
            Arc::new(InMemoryBinanceSpotClient::with_balance(Decimal::from(100))),
            Decimal::new(2, 2),
        );

        let result = adapter.settle(sample_context(100)).await;
        assert_eq!(
            result,
            Err(LiquidityError::InsufficientFunds {
                rail: FundingType::BinanceCex
            })
        );
    }

    #[tokio::test]
    async fn engine_settles_via_binance_adapter() {
        let engine = SettlementEngine::new([
            Arc::new(BinanceCexAdapter::in_memory_mock()) as Arc<dyn SettlementAdapter>,
        ]);

        let receipt = engine
            .settle(FundingType::BinanceCex, sample_context(50))
            .await
            .expect("settled");

        assert_eq!(receipt.rail, FundingType::BinanceCex);
        assert!(receipt.proof.starts_with("CEX-MEM-"));
    }
}
