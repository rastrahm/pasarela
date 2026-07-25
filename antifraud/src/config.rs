//! Configuración del servicio antifraude.

use std::env;

use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// Configuración cargada desde variables de entorno.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub max_amount: Decimal,
    pub blocked_token_hashes: Vec<String>,
    pub decline_score_threshold: f64,
}

impl AppConfig {
    /// Carga configuración desde el entorno.
    pub fn from_env() -> Result<Self> {
        let host = env::var("ANTIFRAUD_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("ANTIFRAUD_PORT")
            .unwrap_or_else(|_| "8082".to_string())
            .parse()
            .context("ANTIFRAUD_PORT inválido")?;

        let api_key = env::var("ANTIFRAUD_API_KEY").context("ANTIFRAUD_API_KEY es obligatoria")?;

        let max_amount = env::var("ANTIFRAUD_MAX_AMOUNT")
            .unwrap_or_else(|_| "50000".to_string())
            .parse::<Decimal>()
            .context("ANTIFRAUD_MAX_AMOUNT inválido")?;

        let blocked_raw = env::var("ANTIFRAUD_BLOCKED_TOKEN_HASHES").unwrap_or_default();
        let blocked_token_hashes = blocked_raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        let decline_score_threshold = env::var("ANTIFRAUD_DECLINE_SCORE_THRESHOLD")
            .unwrap_or_else(|_| "0.85".to_string())
            .parse()
            .context("ANTIFRAUD_DECLINE_SCORE_THRESHOLD inválido")?;

        Ok(Self {
            host,
            port,
            api_key,
            max_amount,
            blocked_token_hashes,
            decline_score_threshold,
        })
    }

    /// Dirección de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
