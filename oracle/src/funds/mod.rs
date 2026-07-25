//! Evaluación de fondos y gestión de holds por riel.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::persistence::hold_store::HoldStore;

/// Tipo de riel de fondeo / liquidación.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    sqlx::Type,
)]
#[sqlx(type_name = "funding_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FundingType {
    TraditionalBank,
    BinanceCex,
    SolanaWallet,
}

impl From<oracle_client::FundingType> for FundingType {
    fn from(value: oracle_client::FundingType) -> Self {
        match value {
            oracle_client::FundingType::TraditionalBank => Self::TraditionalBank,
            oracle_client::FundingType::BinanceCex => Self::BinanceCex,
            oracle_client::FundingType::SolanaWallet => Self::SolanaWallet,
        }
    }
}

impl From<FundingType> for oracle_client::FundingType {
    fn from(value: FundingType) -> Self {
        match value {
            FundingType::TraditionalBank => Self::TraditionalBank,
            FundingType::BinanceCex => Self::BinanceCex,
            FundingType::SolanaWallet => Self::SolanaWallet,
        }
    }
}

/// Estado de evaluación de fondos.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FundStatus {
    pub sufficient: bool,
    pub available_amount: Decimal,
    pub currency: String,
}

/// Hold temporal sobre fondos del riel activo (respuesta de dominio).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HoldSummary {
    pub hold_id: Uuid,
    pub funding_type: FundingType,
    pub amount: Decimal,
    pub currency: String,
}

/// Evalúa disponibilidad de fondos según el riel activo y holds vigentes.
///
/// # Inputs
/// - `rail_provider`: consulta saldo bruto al riel (HTTP/RPC/config).
/// - `config`: spread buffer Binance y parámetros auxiliares.
/// - `hold_store`: store para sumar holds activos.
/// - `amount`: monto solicitado.
/// - `currency`: moneda del pago.
/// - `funding_type`: riel a consultar.
///
/// # Returns
/// `FundStatus` con saldo disponible neto de holds activos.
pub async fn evaluate_funds(
    rail_provider: &dyn crate::rail_adapters::RailBalanceProvider,
    config: &AppConfig,
    hold_store: &dyn HoldStore,
    amount: Decimal,
    currency: &str,
    funding_type: FundingType,
) -> Result<FundStatus, crate::rail_adapters::RailAdapterError> {
    let configured = rail_provider
        .fetch_balance(funding_type, currency)
        .await?;
    let reserved = hold_store
        .sum_active_holds(funding_type)
        .await
        .map_err(|err| crate::rail_adapters::RailAdapterError::Unavailable(err.to_string()))?;

    let gross_available = configured
        .checked_sub(reserved)
        .ok_or_else(|| {
            crate::rail_adapters::RailAdapterError::InvalidResponse(
                "saldo reservado excede balance del riel".into(),
            )
        })?;

    let available = apply_spread_buffer(gross_available, funding_type, config);

    Ok(FundStatus {
        sufficient: available >= amount,
        available_amount: available,
        currency: currency.to_string(),
    })
}

fn apply_spread_buffer(
    balance: Decimal,
    funding_type: FundingType,
    config: &AppConfig,
) -> Decimal {
    match funding_type {
        FundingType::BinanceCex => {
            let factor = Decimal::ONE - config.binance_spread_buffer_pct;
            balance * factor
        }
        _ => balance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::persistence::models::HoldRecord;
    use crate::rail_adapters::MockRailBalanceProvider;
    use std::str::FromStr;

    struct MockHoldStore {
        reserved: Decimal,
    }

    #[async_trait]
    impl HoldStore for MockHoldStore {
        async fn create_hold(
            &self,
            _tx: &mut sqlx::PgTransaction<'_>,
            _hold: crate::persistence::models::CreateHold,
        ) -> Result<HoldRecord, crate::persistence::error::StoreError> {
            unimplemented!()
        }

        async fn get_hold(
            &self,
            _hold_id: Uuid,
        ) -> Result<Option<HoldRecord>, crate::persistence::error::StoreError> {
            unimplemented!()
        }

        async fn release_hold(
            &self,
            _hold_id: Uuid,
        ) -> Result<HoldRecord, crate::persistence::error::StoreError> {
            unimplemented!()
        }

        async fn sum_active_holds(
            &self,
            _funding_type: FundingType,
        ) -> Result<Decimal, crate::persistence::error::StoreError> {
            Ok(self.reserved)
        }

        async fn expire_stale_holds(
            &self,
        ) -> Result<u64, crate::persistence::error::StoreError> {
            Ok(0)
        }

        async fn update_status_in_tx(
            &self,
            _tx: &mut sqlx::PgTransaction<'_>,
            _hold_id: Uuid,
            _status: crate::persistence::models::HoldStatus,
        ) -> Result<HoldRecord, crate::persistence::error::StoreError> {
            unimplemented!()
        }
    }

    fn dec(value: &str) -> Decimal {
        Decimal::from_str(value).expect("decimal")
    }

    fn test_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8081,
            database_url: "postgres://localhost/oracle".to_string(),
            api_key: "test".to_string(),
            allowed_callers: vec![],
            rate_limit_per_minute: 100,
            rail_timeout_secs: 5,
            hold_ttl_secs: 300,
            ttl_cleanup_interval_secs: 60,
            traditional_bank_balance: dec("10000"),
            binance_cex_balance: dec("5000"),
            solana_wallet_balance: dec("2500"),
            binance_spread_buffer_pct: dec("0.02"),
            binance_cex_base_url: "http://127.0.0.1:8083".to_string(),
            binance_cex_api_key: "test-binance-key".to_string(),
            solana_rpc_url: "http://127.0.0.1:8899".to_string(),
            solana_wallet_pubkey: "DemoWallet1111111111111111111111111111111".to_string(),
            solana_token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            antifraud_base_url: "http://127.0.0.1:8082".to_string(),
            antifraud_api_key: "test".to_string(),
            antifraud_timeout_secs: 5,
        }
    }

    #[tokio::test]
    async fn sufficient_funds_for_small_amount() {
        let config = test_config();
        let rail = MockRailBalanceProvider::from_config(&config);
        let store = MockHoldStore {
            reserved: dec("0"),
        };
        let status = evaluate_funds(
            &rail,
            &config,
            &store,
            dec("100"),
            "USD",
            FundingType::TraditionalBank,
        )
        .await
        .expect("evaluate");

        assert!(status.sufficient);
    }

    #[tokio::test]
    async fn insufficient_funds_when_reserved_exceeds_balance() {
        let config = test_config();
        let rail = MockRailBalanceProvider::from_config(&config);
        let store = MockHoldStore {
            reserved: dec("9999"),
        };
        let status = evaluate_funds(
            &rail,
            &config,
            &store,
            dec("100"),
            "USD",
            FundingType::TraditionalBank,
        )
        .await
        .expect("evaluate");

        assert!(!status.sufficient);
    }

    #[tokio::test]
    async fn binance_spread_reduces_available() {
        let config = test_config();
        let rail = MockRailBalanceProvider::from_config(&config);
        let store = MockHoldStore {
            reserved: dec("0"),
        };
        let status = evaluate_funds(
            &rail,
            &config,
            &store,
            dec("4900"),
            "USD",
            FundingType::BinanceCex,
        )
        .await
        .expect("evaluate");

        assert!(status.sufficient);
        assert!(status.available_amount < dec("5000"));

        let status_large = evaluate_funds(
            &rail,
            &config,
            &store,
            dec("4901"),
            "USD",
            FundingType::BinanceCex,
        )
        .await
        .expect("evaluate");

        assert!(!status_large.sufficient);
    }
}
