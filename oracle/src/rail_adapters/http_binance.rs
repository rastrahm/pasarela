//! Cliente HTTP hacia la API Spot simulada de Binance (`binance-sim/`).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::funds::FundingType;

use super::client::RailBalanceProvider;
use super::error::RailAdapterError;

const SPOT_BALANCE_PATH: &str = "/internal/v1/spot/balance";
const API_KEY_HEADER: &str = "x-api-key";

/// Respuesta de `GET /internal/v1/spot/balance`.
#[derive(Debug, Deserialize)]
struct SpotBalanceResponse {
    available: Decimal,
}

/// Consulta saldo Spot simulado vía HTTP.
#[derive(Debug, Clone)]
pub struct HttpBinanceCexProvider {
    http: Client,
    base_url: String,
    api_key: String,
}

impl HttpBinanceCexProvider {
    /// Crea el cliente con URL base, API key y timeout en segundos.
    pub fn new(
        base_url: String,
        api_key: String,
        timeout_secs: u64,
    ) -> Result<Self, RailAdapterError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| RailAdapterError::Unavailable(err.to_string()))?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }
}

#[async_trait]
impl RailBalanceProvider for HttpBinanceCexProvider {
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError> {
        if funding_type != FundingType::BinanceCex {
            return Err(RailAdapterError::Unavailable(format!(
                "HttpBinanceCexProvider no atiende {funding_type:?}"
            )));
        }

        let normalized = normalize_binance_currency(currency)?;
        let url = format!(
            "{}{SPOT_BALANCE_PATH}?currency={normalized}",
            self.base_url
        );

        let response = self
            .http
            .get(&url)
            .header(API_KEY_HEADER, &self.api_key)
            .send()
            .await
            .map_err(map_transport_error)?;

        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            return Err(RailAdapterError::Unavailable(format!(
                "auth rechazada: {}",
                response.status()
            )));
        }

        if !response.status().is_success() {
            return Err(RailAdapterError::Unavailable(format!(
                "status {}",
                response.status()
            )));
        }

        let body = response
            .json::<SpotBalanceResponse>()
            .await
            .map_err(|err| RailAdapterError::InvalidResponse(err.to_string()))?;

        Ok(body.available)
    }
}

fn normalize_binance_currency(currency: &str) -> Result<String, RailAdapterError> {
    let normalized = currency.to_ascii_uppercase();
    match normalized.as_str() {
        "USD" | "USDC" | "USDT" => Ok(normalized),
        other => Err(RailAdapterError::UnsupportedCurrency(other.to_string())),
    }
}

fn map_transport_error(err: reqwest::Error) -> RailAdapterError {
    if err.is_timeout() {
        RailAdapterError::Unavailable("timeout".to_string())
    } else {
        RailAdapterError::Unavailable(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::extract::Query;
    use axum::routing::get;
    use axum::{Json, Router};
    use serde::Deserialize;
    use tokio::net::TcpListener;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct BalanceQuery {
        currency: String,
    }

    #[derive(serde::Serialize)]
    struct BalanceBody {
        currency: String,
        available: Decimal,
    }

    async fn spawn_mock_binance(available: &str) -> String {
        let available = Decimal::from_str(available).expect("decimal");
        let app = Router::new().route(
            SPOT_BALANCE_PATH,
            get(move |Query(query): Query<BalanceQuery>| {
                let available = available;
                async move {
                    Json(BalanceBody {
                        currency: query.currency,
                        available,
                    })
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn http_client_fetches_spot_balance() {
        let base_url = spawn_mock_binance("5000").await;
        let provider =
            HttpBinanceCexProvider::new(base_url, "test-key".to_string(), 2).expect("client");

        let balance = provider
            .fetch_balance(FundingType::BinanceCex, "USDC")
            .await
            .expect("balance");

        assert_eq!(balance, Decimal::from_str("5000").expect("decimal"));
    }

    #[tokio::test]
    async fn http_client_rejects_unsupported_currency() {
        let base_url = spawn_mock_binance("5000").await;
        let provider =
            HttpBinanceCexProvider::new(base_url, "test-key".to_string(), 2).expect("client");

        let result = provider
            .fetch_balance(FundingType::BinanceCex, "EUR")
            .await;

        assert!(matches!(
            result,
            Err(RailAdapterError::UnsupportedCurrency(_))
        ));
    }
}
