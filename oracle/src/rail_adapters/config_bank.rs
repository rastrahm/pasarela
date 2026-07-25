//! Riel bancario tradicional — saldo ficticio desde configuración.

use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::config::AppConfig;
use crate::funds::FundingType;

use super::client::RailBalanceProvider;
use super::error::RailAdapterError;

/// Proveedor de saldo bancario estático (desarrollo / simulación).
#[derive(Debug, Clone)]
pub struct ConfigTraditionalBankProvider {
    config: Arc<AppConfig>,
}

impl ConfigTraditionalBankProvider {
    /// Crea el proveedor con saldo de `TRADITIONAL_BANK_BALANCE`.
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RailBalanceProvider for ConfigTraditionalBankProvider {
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError> {
        if funding_type != FundingType::TraditionalBank {
            return Err(RailAdapterError::Unavailable(format!(
                "ConfigTraditionalBankProvider no atiende {funding_type:?}"
            )));
        }

        validate_currency(currency)?;
        Ok(self.config.traditional_bank_balance)
    }
}

fn validate_currency(currency: &str) -> Result<(), RailAdapterError> {
    let normalized = currency.to_ascii_uppercase();
    if normalized == "USD" {
        Ok(())
    } else {
        Err(RailAdapterError::UnsupportedCurrency(currency.to_string()))
    }
}
