//! Cliente HTTP real hacia `antifraud/`.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use reqwest::StatusCode;

use super::client::AntifraudClient;
use super::error::AntifraudClientError;
use super::types::{ScoreRequest, ScoreResponse};

const SCORE_PATH: &str = "/internal/v1/score";
const API_KEY_HEADER: &str = "x-api-key";

/// Implementación HTTP del cliente antifraude con timeout estricto.
#[derive(Debug, Clone)]
pub struct HttpAntifraudClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl HttpAntifraudClient {
    /// Crea el cliente con URL base, API key y timeout en segundos.
    pub fn new(
        base_url: String,
        api_key: String,
        timeout_secs: u64,
    ) -> Result<Self, AntifraudClientError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| AntifraudClientError::Unavailable(err.to_string()))?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }
}

#[async_trait]
impl AntifraudClient for HttpAntifraudClient {
    async fn score(&self, request: ScoreRequest) -> Result<ScoreResponse, AntifraudClientError> {
        let url = format!("{}{SCORE_PATH}", self.base_url);

        let response = self
            .http
            .post(&url)
            .header(API_KEY_HEADER, &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    AntifraudClientError::Unavailable("timeout".to_string())
                } else {
                    AntifraudClientError::Unavailable(err.to_string())
                }
            })?;

        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            return Err(AntifraudClientError::Unavailable(format!(
                "auth rechazada: {}",
                response.status()
            )));
        }

        if !response.status().is_success() {
            return Err(AntifraudClientError::Unavailable(format!(
                "status {}",
                response.status()
            )));
        }

        let body = response
            .json::<ScoreResponse>()
            .await
            .map_err(|err| AntifraudClientError::InvalidResponse(err.to_string()))?;

        if !body.approved {
            return Err(AntifraudClientError::Declined);
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::routing::post;
    use axum::{Json, Router};
    use rust_decimal::Decimal;
    use tokio::net::TcpListener;
    use uuid::Uuid;

    use super::*;
    use crate::funds::FundingType;

    async fn spawn_mock_antifraud(approved: bool) -> String {
        let app = Router::new().route(
            SCORE_PATH,
            post(move |Json(_body): Json<ScoreRequest>| async move {
                if approved {
                    Json(ScoreResponse::approved())
                } else {
                    Json(ScoreResponse::declined(vec!["amount_too_high".to_string()]))
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

    fn sample_request() -> ScoreRequest {
        ScoreRequest {
            gateway_request_id: Uuid::new_v4(),
            amount: Decimal::from_str("100").expect("decimal"),
            currency: "USD".to_string(),
            funding_type: FundingType::TraditionalBank,
            token_hash: "tok_abc".to_string(),
        }
    }

    #[tokio::test]
    async fn http_client_accepts_approved_score() {
        let base_url = spawn_mock_antifraud(true).await;
        let client = HttpAntifraudClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let response = client.score(sample_request()).await.expect("score");
        assert!(response.approved);
    }

    #[tokio::test]
    async fn http_client_maps_decline_to_error() {
        let base_url = spawn_mock_antifraud(false).await;
        let client = HttpAntifraudClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let result = client.score(sample_request()).await;
        assert!(matches!(result, Err(AntifraudClientError::Declined)));
    }
}
