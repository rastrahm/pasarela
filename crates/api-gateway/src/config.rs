//! Carga de configuración del API Gateway desde variables de entorno.

use std::env;

use anyhow::{Context, Result};

/// Configuración del servicio Gateway.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub oracle_base_url: String,
    pub oracle_api_key: String,
    pub oracle_timeout_secs: u64,
    /// Reservada para persistencia (paso 4.12).
    pub database_url: Option<String>,
}

impl AppConfig {
    /// Carga configuración desde el entorno.
    ///
    /// # Returns
    /// `AppConfig` o error si faltan variables críticas de Oracle.
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

        let database_url = env::var("DATABASE_URL").ok();

        Ok(Self {
            host,
            port,
            oracle_base_url,
            oracle_api_key,
            oracle_timeout_secs,
            database_url,
        })
    }

    /// Dirección de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
        });
    }
}
