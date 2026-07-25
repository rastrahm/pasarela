//! Proveedor en memoria para tests — retorna saldos configurados.

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::config::AppConfig;
use crate::funds::FundingType;

use super::client::RailBalanceProvider;
use super::error::RailAdapterError;

/// Mock que devuelve saldos estáticos sin HTTP ni RPC.
#[derive(Debug, Clone)]
pub struct MockRailBalanceProvider {
    traditional_bank_balance: Decimal,
    binance_cex_balance: Decimal,
    solana_wallet_balance: Decimal,
    unavailable_rail: Option<FundingType>,
}

impl MockRailBalanceProvider {
    /// Crea el mock con saldos de la configuración del Oracle.
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            traditional_bank_balance: config.traditional_bank_balance,
            binance_cex_balance: config.binance_cex_balance,
            solana_wallet_balance: config.solana_wallet_balance,
            unavailable_rail: None,
        }
    }

    /// Simula indisponibilidad de un riel concreto (fail closed).
    pub fn with_unavailable_rail(mut self, funding_type: FundingType) -> Self {
        self.unavailable_rail = Some(funding_type);
        self
    }
}

#[async_trait]
impl RailBalanceProvider for MockRailBalanceProvider {
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError> {
        if self.unavailable_rail == Some(funding_type) {
            return Err(RailAdapterError::Unavailable(format!(
                "mock: riel {funding_type:?} no disponible"
            )));
        }

        let _ = currency;
        let balance = match funding_type {
            FundingType::TraditionalBank => self.traditional_bank_balance,
            FundingType::BinanceCex => self.binance_cex_balance,
            FundingType::SolanaWallet => self.solana_wallet_balance,
        };

        Ok(balance)
    }
}
