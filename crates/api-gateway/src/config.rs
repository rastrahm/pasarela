//! Carga de configuración del API Gateway desde variables de entorno.

use std::env;

use anyhow::{Context, Result};
use domain::{FundingType, MerchantId};
use rail_switcher::RailConfig;
use uuid::Uuid;

use crate::services::merchant::{MerchantApiKeyEntry, parse_merchant_api_keys};
use crate::services::rails::{default_availability, default_rail_configs, RailContext};

/// Configuración del servicio Gateway.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub oracle_base_url: String,
    pub oracle_api_key: String,
    pub oracle_timeout_secs: u64,
    /// Verifica `GET /health` del Oracle al arranque.
    pub oracle_health_check: bool,
    /// Comercio por defecto cuando se usa `GATEWAY_TEST_API_KEY`.
    pub default_merchant_id: MerchantId,
    /// Pares `sk_*:merchant_uuid` desde `GATEWAY_MERCHANT_API_KEYS`.
    pub merchant_api_keys: Vec<MerchantApiKeyEntry>,
    /// Atajo dev: una sola API key de prueba (`sk_test_...`).
    pub bootstrap_test_api_key: Option<String>,
    /// Preferencia de riel del comercio cuando el checkout no la indica.
    pub merchant_default_funding_type: Option<FundingType>,
    /// Habilita fallback automático D3 en Rail Switcher.
    pub rail_fallback_enabled: bool,
    /// Configuración operativa de rieles (`RAIL_CONFIG`).
    pub rail_configs: Vec<RailConfig>,
    /// Reservada para persistencia PostgreSQL (paso 4.12).
    pub database_url: Option<String>,
}

impl AppConfig {
    /// Carga configuración desde el entorno.
    pub fn from_env() -> Result<Self> {
        let host = env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("GATEWAY_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("GATEWAY_PORT debe ser un entero válido")?;

        let oracle_base_url = env::var("ORACLE_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
        let oracle_api_key =
            env::var("ORACLE_API_KEY").context("ORACLE_API_KEY es obligatoria")?;

        let oracle_timeout_secs = env::var("ORACLE_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("ORACLE_TIMEOUT_SECS inválido")?;

        let oracle_health_check = parse_bool_env("GATEWAY_ORACLE_HEALTH_CHECK", true);

        let default_merchant_id = env::var("GATEWAY_DEFAULT_MERCHANT_ID")
            .ok()
            .and_then(|raw| Uuid::parse_str(&raw).ok())
            .map(MerchantId::new)
            .unwrap_or_else(|| MerchantId::new(Uuid::new_v4()));

        let merchant_api_keys = env::var("GATEWAY_MERCHANT_API_KEYS")
            .ok()
            .map(|raw| parse_merchant_api_keys(&raw))
            .transpose()
            .context("GATEWAY_MERCHANT_API_KEYS inválido")?
            .unwrap_or_default();

        let bootstrap_test_api_key = env::var("GATEWAY_TEST_API_KEY").ok();

        let merchant_default_funding_type = env::var("GATEWAY_DEFAULT_FUNDING_TYPE")
            .ok()
            .and_then(|raw| parse_funding_type(&raw));

        let rail_fallback_enabled = parse_bool_env("GATEWAY_RAIL_FALLBACK_ENABLED", true);
        let rail_configs = load_rail_configs();

        let database_url = env::var("DATABASE_URL").ok();

        Ok(Self {
            host,
            port,
            oracle_base_url,
            oracle_api_key,
            oracle_timeout_secs,
            oracle_health_check,
            default_merchant_id,
            merchant_api_keys,
            bootstrap_test_api_key,
            merchant_default_funding_type,
            rail_fallback_enabled,
            rail_configs,
            database_url,
        })
    }

    /// Construye el contexto operativo del Rail Switcher a partir de la config.
    pub fn rail_context(&self) -> RailContext {
        RailContext {
            configs: self.rail_configs.clone(),
            availability: default_availability(),
            merchant_default: self.merchant_default_funding_type,
            fallback_enabled: self.rail_fallback_enabled,
        }
    }

    /// Dirección de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Construye el registro de comercios desde la configuración cargada.
    pub fn build_merchant_registry(
        &self,
    ) -> Result<crate::services::merchant::MerchantRegistry, crate::services::merchant::MerchantRegistryError>
    {
        use crate::services::merchant::{ApiKeyMode, MerchantRegistry, MerchantRegistryError};

        let registry = MerchantRegistry::default();

        if !self.merchant_api_keys.is_empty() {
            for entry in &self.merchant_api_keys {
                registry.register(
                    entry.api_key.clone(),
                    entry.merchant_id,
                    entry.mode,
                );
            }
            return Ok(registry);
        }

        if let Some(api_key) = &self.bootstrap_test_api_key {
            registry.register(api_key.clone(), self.default_merchant_id, ApiKeyMode::Test);
            return Ok(registry);
        }

        Err(MerchantRegistryError::MissingMerchantKeys)
    }
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|raw| matches!(raw.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(default)
}

fn parse_funding_type(raw: &str) -> Option<FundingType> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "traditional_bank" => Some(FundingType::TraditionalBank),
        "binance_cex" => Some(FundingType::BinanceCex),
        "solana_wallet" => Some(FundingType::SolanaWallet),
        _ => None,
    }
}

fn load_rail_configs() -> Vec<RailConfig> {
    let mut configs = default_rail_configs();
    for config in &mut configs {
        let key = format!(
            "GATEWAY_RAIL_{}_ENABLED",
            funding_type_env_suffix(config.funding_type)
        );
        if let Ok(raw) = env::var(&key) {
            config.enabled = matches!(raw.to_ascii_lowercase().as_str(), "1" | "true" | "yes");
        }
    }
    configs
}

fn funding_type_env_suffix(funding_type: FundingType) -> &'static str {
    match funding_type {
        FundingType::TraditionalBank => "TRADITIONAL_BANK",
        FundingType::BinanceCex => "BINANCE_CEX",
        FundingType::SolanaWallet => "SOLANA_WALLET",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env(key: &str, value: &str, f: impl FnOnce()) {
        unsafe { env::set_var(key, value) };
        f();
        unsafe { env::remove_var(key) };
    }

    fn clear_optional_gateway_env() {
        for key in [
            "GATEWAY_DEFAULT_FUNDING_TYPE",
            "GATEWAY_RAIL_TRADITIONAL_BANK_ENABLED",
            "GATEWAY_RAIL_BINANCE_CEX_ENABLED",
            "GATEWAY_RAIL_SOLANA_WALLET_ENABLED",
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    #[test]
    fn from_env_requires_oracle_api_key() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        unsafe { env::remove_var("ORACLE_API_KEY") };
        assert!(AppConfig::from_env().is_err());
    }

    #[test]
    fn from_env_loads_defaults() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        with_env("ORACLE_API_KEY", "test-key", || {
            let config = AppConfig::from_env().expect("config");
            assert_eq!(config.port, 8080);
            assert_eq!(config.oracle_base_url, "http://127.0.0.1:8081");
            assert_eq!(config.oracle_api_key, "test-key");
            assert!(config.oracle_health_check);
            assert!(config.rail_fallback_enabled);
        });
    }

    #[test]
    fn from_env_parses_merchant_default_funding_type() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        with_env("ORACLE_API_KEY", "test-key", || {
            with_env("GATEWAY_DEFAULT_FUNDING_TYPE", "binance_cex", || {
                let config = AppConfig::from_env().expect("config");
                assert_eq!(
                    config.merchant_default_funding_type,
                    Some(FundingType::BinanceCex)
                );
            });
        });
    }

    #[test]
    fn rail_context_reflects_merchant_default() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        with_env("ORACLE_API_KEY", "test-key", || {
            with_env("GATEWAY_DEFAULT_FUNDING_TYPE", "solana_wallet", || {
                let config = AppConfig::from_env().expect("config");
                let context = config.rail_context();
                assert_eq!(
                    context.merchant_default,
                    Some(FundingType::SolanaWallet)
                );
            });
        });
    }

    #[test]
    fn load_rail_configs_respects_enabled_flags() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        with_env("ORACLE_API_KEY", "test-key", || {
            with_env("GATEWAY_RAIL_TRADITIONAL_BANK_ENABLED", "false", || {
                let config = AppConfig::from_env().expect("config");
                let bank = config
                    .rail_configs
                    .iter()
                    .find(|entry| entry.funding_type == FundingType::TraditionalBank)
                    .expect("bank config");
                assert!(!bank.enabled);
            });
        });
    }

    #[test]
    fn build_merchant_registry_from_test_api_key() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        clear_optional_gateway_env();
        with_env("ORACLE_API_KEY", "test-key", || {
            with_env("GATEWAY_TEST_API_KEY", "sk_test_validkey1", || {
                let config = AppConfig::from_env().expect("config");
                let registry = config.build_merchant_registry().expect("registry");
                assert!(registry.authenticate("sk_test_validkey1").is_ok());
            });
        });
    }
}
