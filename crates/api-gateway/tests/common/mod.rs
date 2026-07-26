//! Helpers compartidos para tests de integración del Gateway.

#![allow(dead_code)]

mod http;
mod mock_oracle;

use std::sync::Arc;

use domain::MerchantId;
use oracle_client::HttpOracleClient;
use rail_switcher::RailSwitcher;
use settlement_adapters::SettlementEngine;
use uuid::Uuid;

use api_gateway::config::AppConfig;
use api_gateway::services::{MerchantRegistry, RailContext};
use api_gateway::{build_app, state::AppState};

pub use http::{
    checkout_payload, checkout_payload_no_rail, get_health, get_transaction,
    post_checkout, post_checkout_unauthenticated, post_checkout_without_idempotency,
};
pub use mock_oracle::{spawn_mock_oracle, MockOracleOpts, OracleCapture};

/// API key alineada con `oracle_authorization::test_support::TEST_ORACLE_API_KEY`.
pub const REAL_ORACLE_API_KEY: &str = oracle_authorization::test_support::TEST_ORACLE_API_KEY;

/// Configuración del Gateway apuntando a un Oracle real en tests cross-service.
pub fn cross_service_app_config(oracle_base_url: String, merchant_id: MerchantId) -> Arc<AppConfig> {
    Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url,
        oracle_api_key: REAL_ORACLE_API_KEY.to_string(),
        oracle_timeout_secs: 5,
        oracle_health_check: false,
        default_merchant_id: merchant_id,
        merchant_api_keys: vec![],
        bootstrap_test_api_key: None,
        merchant_default_funding_type: None,
        rail_fallback_enabled: true,
        rail_configs: api_gateway::services::rails::default_rail_configs(),
        database_url: None,
    })
}

/// App Axum con Oracle HTTP real (proceso local) y adaptadores stub de liquidación.
pub fn build_cross_service_app(oracle_base_url: String) -> axum::Router {
    let merchant_id = test_merchant_id();
    let config = cross_service_app_config(oracle_base_url.clone(), merchant_id);
    let oracle_client = Arc::new(
        HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("cliente oracle real"),
    );

    build_app(AppState::from_parts(
        config,
        oracle_client,
        SettlementEngine::with_stub_adapters(),
        RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    ))
}

/// API key de comercio válida para tests (`sk_test_` + 8+ chars).
pub const TEST_API_KEY: &str = "sk_test_validkey1";

pub fn test_merchant_id() -> MerchantId {
    MerchantId::new(Uuid::new_v4())
}

pub fn test_merchant_registry(merchant_id: MerchantId) -> Arc<MerchantRegistry> {
    Arc::new(MerchantRegistry::single(TEST_API_KEY, merchant_id))
}

pub fn test_app_config(oracle_base_url: String, merchant_id: MerchantId) -> Arc<AppConfig> {
    Arc::new(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        oracle_base_url,
        oracle_api_key: "test-key".to_string(),
        oracle_timeout_secs: 2,
        oracle_health_check: false,
        default_merchant_id: merchant_id,
        merchant_api_keys: vec![],
        bootstrap_test_api_key: None,
        merchant_default_funding_type: None,
        rail_fallback_enabled: true,
        rail_configs: api_gateway::services::rails::default_rail_configs(),
        database_url: None,
    })
}

pub fn bearer_header() -> (&'static str, String) {
    ("authorization", format!("Bearer {TEST_API_KEY}"))
}

/// Construye `AppState` con mock Oracle HTTP y stub adapters (mock rieles).
pub fn build_test_state(
    oracle_base_url: String,
    rail_context: RailContext,
    settlement_engine: SettlementEngine,
) -> AppState {
    let merchant_id = test_merchant_id();
    let config = test_app_config(oracle_base_url, merchant_id);
    let oracle_client = Arc::new(
        HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("client"),
    );

    AppState::from_parts(
        config,
        oracle_client,
        settlement_engine,
        RailSwitcher,
        rail_context,
        test_merchant_registry(merchant_id),
    )
}

/// App Axum lista para tests E2E (Gateway + mock Oracle + mock rieles).
pub fn build_test_app(
    oracle_base_url: String,
    rail_context: RailContext,
    settlement_engine: SettlementEngine,
) -> axum::Router {
    build_app(build_test_state(
        oracle_base_url,
        rail_context,
        settlement_engine,
    ))
}

/// App con RailContext por defecto y adaptadores stub de los tres rieles.
pub fn build_default_test_app(oracle_base_url: String) -> axum::Router {
    build_test_app(
        oracle_base_url,
        RailContext::default(),
        SettlementEngine::with_stub_adapters(),
    )
}
