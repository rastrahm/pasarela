//! Implementación HTTP del cliente Oracle.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};

use crate::client::{OracleClient, RequestOptions};
use crate::dto::{
    AuthorizeRequest, AuthorizeResponse, ErrorResponse, HealthResponse, ReleaseHoldRequest,
    ReleaseHoldResponse,
};
use crate::error::OracleClientError;

const HEALTH_PATH: &str = "/health";
const AUTHORIZE_PATH: &str = "/internal/v1/authorize";
const RELEASE_HOLD_PATH: &str = "/internal/v1/hold/release";
const API_KEY_HEADER: &str = "x-api-key";
const FORWARDED_FOR_HEADER: &str = "x-forwarded-for";

/// Cliente HTTP real hacia el microservicio Oracle.
#[derive(Debug, Clone)]
pub struct HttpOracleClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl HttpOracleClient {
    /// Crea el cliente con URL base, API key y timeout en segundos.
    pub fn new(
        base_url: String,
        api_key: String,
        timeout_secs: u64,
    ) -> Result<Self, OracleClientError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| OracleClientError::Unavailable(err.to_string()))?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    fn authed_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.base_url))
            .header(API_KEY_HEADER, &self.api_key)
    }

    fn apply_forwarded_for(builder: RequestBuilder, options: &RequestOptions) -> RequestBuilder {
        if let Some(ip) = &options.caller_ip {
            builder.header(FORWARDED_FOR_HEADER, ip)
        } else {
            builder
        }
    }

    async fn parse_error_response(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, OracleClientError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response.json::<ErrorResponse>().await.ok();
        Err(OracleClientError::from_http_status(status, body))
    }
}

#[async_trait]
impl OracleClient for HttpOracleClient {
    async fn health(&self) -> Result<HealthResponse, OracleClientError> {
        let response = self
            .http
            .get(format!("{}{HEALTH_PATH}", self.base_url))
            .send()
            .await
            .map_err(map_transport_error)?;

        let response = Self::parse_error_response(response).await?;
        response
            .json::<HealthResponse>()
            .await
            .map_err(|err| OracleClientError::InvalidResponse(err.to_string()))
    }

    async fn authorize(
        &self,
        request: AuthorizeRequest,
        options: RequestOptions,
    ) -> Result<AuthorizeResponse, OracleClientError> {
        let builder = Self::apply_forwarded_for(
            self.authed_request(reqwest::Method::POST, AUTHORIZE_PATH).json(&request),
            &options,
        );

        let response = builder.send().await.map_err(map_transport_error)?;
        let response = Self::parse_error_response(response).await?;

        response
            .json::<AuthorizeResponse>()
            .await
            .map_err(|err| OracleClientError::InvalidResponse(err.to_string()))
    }

    async fn release_hold_request(
        &self,
        request: ReleaseHoldRequest,
        options: RequestOptions,
    ) -> Result<ReleaseHoldResponse, OracleClientError> {
        let builder = Self::apply_forwarded_for(
            self.authed_request(reqwest::Method::POST, RELEASE_HOLD_PATH).json(&request),
            &options,
        );

        let response = builder.send().await.map_err(map_transport_error)?;
        let response = Self::parse_error_response(response).await?;

        response
            .json::<ReleaseHoldResponse>()
            .await
            .map_err(|err| OracleClientError::InvalidResponse(err.to_string()))
    }
}

fn map_transport_error(err: reqwest::Error) -> OracleClientError {
    if err.is_timeout() {
        OracleClientError::Unavailable("timeout".to_string())
    } else {
        OracleClientError::Unavailable(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::routing::{get, post};
    use axum::{Json, Router};
    use tokio::net::TcpListener;
    use uuid::Uuid;

    use super::*;
    use crate::dto::{CardPayload, FundingType, error_codes};

    async fn spawn_mock_oracle() -> String {
        let app = Router::new()
            .route("/health", get(|| async {
                Json(HealthResponse {
                    status: "ok".to_string(),
                    service: "oracle-authorization".to_string(),
                })
            }))
            .route(
                AUTHORIZE_PATH,
                post(|Json(body): Json<AuthorizeRequest>| async move {
                    let response: axum::response::Response = if body.amount >= 10_000.0 {
                        (
                            StatusCode::PAYMENT_REQUIRED,
                            Json(ErrorResponse {
                                error_code: error_codes::INSUFFICIENT_FUNDS.to_string(),
                                message: "fondos insuficientes".to_string(),
                            }),
                        )
                            .into_response()
                    } else {
                        (
                            StatusCode::OK,
                            Json(AuthorizeResponse {
                                hold_id: Uuid::new_v4(),
                                brand: "visa".to_string(),
                                brand_code: 1,
                                last_four: "1111".to_string(),
                                gateway_request_id: body.gateway_request_id,
                            }),
                        )
                            .into_response()
                    };
                    response
                }),
            )
            .route(
                RELEASE_HOLD_PATH,
                post(|Json(body): Json<ReleaseHoldRequest>| async move {
                    (
                        StatusCode::OK,
                        Json(ReleaseHoldResponse {
                            hold_id: body.hold_id,
                            status: "released".to_string(),
                        }),
                    )
                }),
            );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        format!("http://{addr}")
    }

    fn sample_authorize_request() -> AuthorizeRequest {
        AuthorizeRequest {
            gateway_request_id: Uuid::new_v4(),
            card: CardPayload {
                pan: "4111111111111111".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "30".to_string(),
                cvv: "123".to_string(),
                cardholder: "Demo".to_string(),
            },
            amount: 100.0,
            currency: "USD".to_string(),
            funding_type: FundingType::TraditionalBank,
        }
    }

    #[tokio::test]
    async fn http_client_fetches_health() {
        let base_url = spawn_mock_oracle().await;
        let client = HttpOracleClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let health = client.health().await.expect("health");
        assert_eq!(health.status, "ok");
    }

    #[tokio::test]
    async fn http_client_authorizes_successfully() {
        let base_url = spawn_mock_oracle().await;
        let client = HttpOracleClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let request = sample_authorize_request();
        let gateway_id = request.gateway_request_id;
        let response = client
            .authorize(
                request,
                RequestOptions {
                    caller_ip: Some("127.0.0.1".to_string()),
                },
            )
            .await
            .expect("authorize");

        assert_eq!(response.gateway_request_id, gateway_id);
        assert_eq!(response.brand_code, 1);
    }

    #[tokio::test]
    async fn http_client_maps_insufficient_funds() {
        let base_url = spawn_mock_oracle().await;
        let client = HttpOracleClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let mut request = sample_authorize_request();
        request.amount = 50_000.0;

        let result = client.authorize(request, RequestOptions::default()).await;
        assert_eq!(result, Err(OracleClientError::InsufficientFunds));
    }

    #[tokio::test]
    async fn http_client_releases_hold() {
        let base_url = spawn_mock_oracle().await;
        let client = HttpOracleClient::new(base_url, "test-key".to_string(), 2).expect("client");

        let hold_id = Uuid::new_v4();
        let response = client.release_hold(hold_id, RequestOptions::default()).await.expect("release");

        assert_eq!(response.hold_id, hold_id);
        assert_eq!(response.status, "released");
    }
}
