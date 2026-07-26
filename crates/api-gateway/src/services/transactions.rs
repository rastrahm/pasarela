//! Consulta de transacciones — UC-09.

use uuid::Uuid;

use crate::error::GatewayError;
use crate::routes::TransactionResponse;
use crate::state::AppState;

/// Busca una transacción por ID en el repositorio in-memory del Gateway.
pub fn lookup_transaction(
    state: &AppState,
    transaction_id: Uuid,
) -> Result<TransactionResponse, GatewayError> {
    let record = state
        .get_transaction(transaction_id)
        .ok_or(GatewayError::NotFound)?;

    Ok(TransactionResponse {
        transaction_id: record.transaction_id,
        status: record.status,
        rail_used: record.rail_used,
        settlement_proof: record.settlement_proof,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use domain::{FundingType, MerchantId, TransactionStatus};
    use oracle_client::{
        AuthorizeResponse, HealthResponse, OracleClient, OracleClientError, ReleaseHoldResponse,
    };
    use rail_switcher::RailSwitcher;
    use settlement_adapters::SettlementEngine;
    use uuid::Uuid;

    use super::*;
    use crate::config::AppConfig;
    use crate::services::rails::default_rail_configs;
    use crate::services::{MerchantRegistry, RailContext};
    use crate::state::{AppState, TransactionRecord};

    struct StubOracle;

    #[async_trait]
    impl OracleClient for StubOracle {
        async fn health(&self) -> Result<HealthResponse, OracleClientError> {
            Ok(HealthResponse {
                status: "ok".to_string(),
                service: "stub".to_string(),
            })
        }

        async fn authorize(
            &self,
            _request: oracle_client::AuthorizeRequest,
            _options: oracle_client::RequestOptions,
        ) -> Result<AuthorizeResponse, OracleClientError> {
            Err(OracleClientError::InvalidCard)
        }

        async fn release_hold_request(
            &self,
            _request: oracle_client::ReleaseHoldRequest,
            _options: oracle_client::RequestOptions,
        ) -> Result<ReleaseHoldResponse, OracleClientError> {
            Ok(ReleaseHoldResponse {
                hold_id: Uuid::new_v4(),
                status: "released".to_string(),
            })
        }
    }

    fn test_state() -> AppState {
        let merchant_id = MerchantId::new(Uuid::new_v4());
        let config = Arc::new(AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            oracle_base_url: "http://mock".to_string(),
            oracle_api_key: "key".to_string(),
            oracle_timeout_secs: 2,
            oracle_health_check: false,
            default_merchant_id: merchant_id,
            merchant_api_keys: vec![],
            bootstrap_test_api_key: None,
            merchant_default_funding_type: None,
            rail_fallback_enabled: true,
            rail_configs: default_rail_configs(),
            database_url: None,
        });

        AppState::from_parts(
            config,
            Arc::new(StubOracle),
            SettlementEngine::with_stub_adapters(),
            RailSwitcher,
            RailContext::default(),
            Arc::new(MerchantRegistry::single("sk_test_validkey1", merchant_id)),
        )
    }

    #[test]
    fn returns_transaction_when_found() {
        let state = test_state();
        let tx_id = Uuid::new_v4();
        state.store_transaction(TransactionRecord {
            transaction_id: tx_id,
            status: TransactionStatus::Settled,
            rail_used: Some(FundingType::TraditionalBank),
            settlement_proof: Some("ACH-TEST".to_string()),
        });

        let response = lookup_transaction(&state, tx_id).expect("found");
        assert_eq!(response.transaction_id, tx_id);
        assert_eq!(response.status, TransactionStatus::Settled);
        assert_eq!(response.rail_used, Some(FundingType::TraditionalBank));
        assert_eq!(response.settlement_proof.as_deref(), Some("ACH-TEST"));
    }

    #[test]
    fn returns_not_found_for_unknown_id() {
        let state = test_state();
        let err = lookup_transaction(&state, Uuid::new_v4()).expect_err("missing");
        assert!(matches!(err, GatewayError::NotFound));
    }
}
