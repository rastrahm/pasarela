//! Carga de configuración del API Gateway desde variables de entorno.

use std::env;

use anyhow::{Context, Result};
use domain::{FundingType, MerchantId};
use rail_switcher::RailConfig;
use uuid::Uuid;

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
    /// Comercio por defecto hasta auth API key (paso 4.11).
    pub default_merchant_id: MerchantId,
    /// Preferencia de riel del comercio cuando el checkout no la indica.
    pub merchant_default_funding_type: Option<FundingType>,
    /// Habilita fallback automático D3 en Rail Switcher.
    pub rail_fallback_enabled: bool,
    /// Configuración operativa de rieles (`RAIL_CONFIG`).
    pub rail_configs: Vec<RailConfig>,
    /// Reservada para persistencia (paso 4.12).
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

    fn with_env(key: &str, value: &str, f: impl FnOnce()) {
        unsafe { env::set_var(key, value) };
        f();
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn from_env_requires_oracle_api_key() {
        unsafe { env::remove_var("ORACLE_API_KEY") };
        assert!(AppConfig::from_env().is_err());
    }

    #[test]
    fn from_env_loads_defaults() {
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
}
