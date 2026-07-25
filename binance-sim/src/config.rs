//! Configuración del simulador Binance Spot.

use std::collections::HashMap;
use std::env;

use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// Configuración cargada desde variables de entorno.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub default_balance: Decimal,
    pub balances_by_currency: HashMap<String, Decimal>,
}

impl AppConfig {
    /// Carga configuración desde el entorno.
    pub fn from_env() -> Result<Self> {
        let host = env::var("BINANCE_SIM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("BINANCE_SIM_PORT")
            .unwrap_or_else(|_| "8083".to_string())
            .parse()
            .context("BINANCE_SIM_PORT inválido")?;

        let api_key = env::var("BINANCE_CEX_API_KEY")
            .context("BINANCE_CEX_API_KEY es obligatoria")?;

        let default_balance = env::var("BINANCE_CEX_BALANCE")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<Decimal>()
            .context("BINANCE_CEX_BALANCE inválido")?;

        let mut balances_by_currency = HashMap::new();
        if let Ok(raw) = env::var("BINANCE_CEX_BALANCES") {
            for entry in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let (currency, amount) = entry
                    .split_once('=')
                    .with_context(|| format!("entrada inválida en BINANCE_CEX_BALANCES: {entry}"))?;
                let parsed = amount
                    .parse::<Decimal>()
                    .with_context(|| format!("balance inválido para {currency}"))?;
                balances_by_currency.insert(currency.to_ascii_uppercase(), parsed);
            }
        }

        Ok(Self {
            host,
            port,
            api_key,
            default_balance,
            balances_by_currency,
        })
    }

    /// Dirección de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Saldo Spot disponible para una moneda.
    pub fn balance_for(&self, currency: &str) -> Decimal {
        let normalized = currency.to_ascii_uppercase();
        self.balances_by_currency
            .get(&normalized)
            .copied()
            .unwrap_or(self.default_balance)
    }
}
