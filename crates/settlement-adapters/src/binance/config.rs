//! Configuración del adaptador Binance CEX.

use std::env;

use rust_decimal::Decimal;
use thiserror::Error;

/// Parámetros de conexión y spread buffer (decisión D4).
#[derive(Debug, Clone, PartialEq)]
pub struct BinanceCexConfig {
    /// URL base del simulador (`binance-sim/`) o API real.
    pub base_url: String,
    /// API key compartida con el servicio CEX.
    pub api_key: String,
    /// Porcentaje de spread buffer (ej. `0.02` = 2%).
    pub spread_buffer_pct: Decimal,
    /// Timeout HTTP en segundos.
    pub timeout_secs: u64,
}

impl BinanceCexConfig {
    /// Carga configuración desde variables de entorno estándar del monorepo.
    ///
    /// Variables: `BINANCE_CEX_BASE_URL`, `BINANCE_CEX_API_KEY`, `BINANCE_SPREAD_BUFFER_PCT`.
    pub fn from_env() -> Result<Self, BinanceConfigError> {
        let base_url = env::var("BINANCE_CEX_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8083".to_string());
        let api_key =
            env::var("BINANCE_CEX_API_KEY").map_err(|_| BinanceConfigError::MissingApiKey)?;

        let spread_buffer_pct = env::var("BINANCE_SPREAD_BUFFER_PCT")
            .unwrap_or_else(|_| "0.02".to_string())
            .parse::<Decimal>()
            .map_err(|_| BinanceConfigError::InvalidSpreadBuffer)?;

        let timeout_secs = env::var("BINANCE_CEX_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| BinanceConfigError::InvalidTimeout)?;

        Ok(Self {
            base_url,
            api_key,
            spread_buffer_pct,
            timeout_secs,
        })
    }

    /// Configuración para tests con simulador en memoria o mock HTTP.
    pub fn for_testing(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            spread_buffer_pct: Decimal::new(2, 2),
            timeout_secs: 5,
        }
    }
}

/// Error al cargar configuración Binance.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BinanceConfigError {
    #[error("BINANCE_CEX_API_KEY es obligatoria")]
    MissingApiKey,

    #[error("BINANCE_SPREAD_BUFFER_PCT inválido")]
    InvalidSpreadBuffer,

    #[error("BINANCE_CEX_TIMEOUT_SECS inválido")]
    InvalidTimeout,

    #[error("no se pudo inicializar cliente HTTP Binance")]
    ClientInitFailed,
}
