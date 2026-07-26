//! Helpers compartidos para tests de integración del Gateway.

use std::sync::Arc;

use domain::MerchantId;
use uuid::Uuid;

use api_gateway::config::AppConfig;
use api_gateway::services::MerchantRegistry;

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
