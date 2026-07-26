//! Cliente HTTP hacia la API Spot simulada de Binance.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::config::BinanceCexConfig;

const SPOT_DEBIT_PATH: &str = "/internal/v1/spot/debit";
const API_KEY_HEADER: &str = "x-api-key";

/// Solicitud de débito Spot (UC-06).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotDebitRequest {
    pub amount: Decimal,
    pub currency: String,
    pub client_order_id: String,
    pub spread_buffer_pct: Decimal,
}

/// Respuesta de débito Spot simulado.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpotDebitResponse {
    pub order_id: String,
    pub status: String,
    pub debited_amount: Decimal,
    pub currency: String,
}

/// Cliente de débito Spot — HTTP o en memoria.
#[async_trait]
pub trait BinanceSpotClient: Send + Sync {
    /// Ejecuta débito custodial Spot con validación de spread buffer.
    async fn debit(&self, request: SpotDebitRequest) -> Result<SpotDebitResponse, BinanceClientError>;
}

/// Cliente HTTP hacia `binance-sim/` (`POST /internal/v1/spot/debit`).
#[derive(Debug, Clone)]
pub struct HttpBinanceSpotClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl HttpBinanceSpotClient {
    /// Crea el cliente a partir de [`BinanceCexConfig`].
    pub fn new(config: &BinanceCexConfig) -> Result<Self, BinanceClientError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|err| BinanceClientError::Unavailable(err.to_string()))?;

        Ok(Self {
            http,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key: config.api_key.clone(),
        })
    }
}

#[async_trait]
impl BinanceSpotClient for HttpBinanceSpotClient {
    async fn debit(&self, request: SpotDebitRequest) -> Result<SpotDebitResponse, BinanceClientError> {
        let url = format!("{}{SPOT_DEBIT_PATH}", self.base_url);

        let response = self
            .http
            .post(&url)
            .header(API_KEY_HEADER, &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(map_transport_error)?;

        if response.status() == StatusCode::PAYMENT_REQUIRED {
            return Err(BinanceClientError::InsufficientFunds);
        }

        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            return Err(BinanceClientError::Unavailable(format!(
                "auth rechazada: {}",
                response.status()
            )));
        }

        if !response.status().is_success() {
            return Err(BinanceClientError::Unavailable(format!(
                "status {}",
                response.status()
            )));
        }

        response
            .json::<SpotDebitResponse>()
            .await
            .map_err(|err| BinanceClientError::InvalidResponse(err.to_string()))
    }
}

/// Cliente en memoria para tests unitarios sin levantar `binance-sim`.
#[derive(Debug)]
pub struct InMemoryBinanceSpotClient {
    balances: std::sync::Mutex<std::collections::HashMap<String, Decimal>>,
}

impl InMemoryBinanceSpotClient {
    /// Crea un cliente con saldo inicial uniforme en USD/USDC/USDT.
    pub fn with_balance(balance: Decimal) -> Self {
        let mut balances = std::collections::HashMap::new();
        for currency in ["USD", "USDC", "USDT"] {
            balances.insert(currency.to_string(), balance);
        }

        Self {
            balances: std::sync::Mutex::new(balances),
        }
    }
}

#[async_trait]
impl BinanceSpotClient for InMemoryBinanceSpotClient {
    async fn debit(&self, request: SpotDebitRequest) -> Result<SpotDebitResponse, BinanceClientError> {
        if request.amount <= Decimal::ZERO {
            return Err(BinanceClientError::InvalidRequest);
        }

        let normalized = request.currency.to_ascii_uppercase();
        if !matches!(normalized.as_str(), "USD" | "USDC" | "USDT") {
            return Err(BinanceClientError::UnsupportedCurrency);
        }

        let mut balances = self
            .balances
            .lock()
            .map_err(|_| BinanceClientError::Unavailable("lock poisoned".to_string()))?;

        let balance = balances.get(&normalized).copied().unwrap_or(Decimal::ZERO);
        let factor = Decimal::ONE - request.spread_buffer_pct;
        let effective = balance * factor;

        if balance < request.amount || effective < request.amount {
            return Err(BinanceClientError::InsufficientFunds);
        }

        let remaining = balance - request.amount;
        balances.insert(normalized.clone(), remaining);

        Ok(SpotDebitResponse {
            order_id: format!("CEX-MEM-{}", request.client_order_id),
            status: "filled".to_string(),
            debited_amount: request.amount,
            currency: normalized,
        })
    }
}

/// Errores del cliente Binance Spot.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BinanceClientError {
    #[error("fondos insuficientes en Spot")]
    InsufficientFunds,

    #[error("moneda no soportada")]
    UnsupportedCurrency,

    #[error("solicitud inválida")]
    InvalidRequest,

    #[error("riel no disponible: {0}")]
    Unavailable(String),

    #[error("respuesta inválida: {0}")]
    InvalidResponse(String),
}

fn map_transport_error(err: reqwest::Error) -> BinanceClientError {
    if err.is_timeout() {
        BinanceClientError::Unavailable("timeout".to_string())
    } else {
        BinanceClientError::Unavailable(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use super::*;

    fn debit_request(amount: &str) -> SpotDebitRequest {
        SpotDebitRequest {
            amount: Decimal::from_str(amount).expect("decimal"),
            currency: "USDC".to_string(),
            client_order_id: "hold-123".to_string(),
            spread_buffer_pct: Decimal::new(2, 2),
        }
    }

    #[tokio::test]
    async fn in_memory_client_debits_with_spread_buffer() {
        let client = InMemoryBinanceSpotClient::with_balance(Decimal::from(5000));
        let response = client.debit(debit_request("100")).await.expect("debit");

        assert_eq!(response.status, "filled");
        assert!(response.order_id.starts_with("CEX-MEM-"));
        assert_eq!(response.debited_amount, Decimal::from(100));
    }

    #[tokio::test]
    async fn in_memory_client_rejects_when_spread_constrains_balance() {
        let client = InMemoryBinanceSpotClient::with_balance(Decimal::from(100));
        let result = client.debit(debit_request("100")).await;

        assert_eq!(result, Err(BinanceClientError::InsufficientFunds));
    }
}
