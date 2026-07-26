//! Idempotencia de checkout — header `Idempotency-Key` obligatorio (D9).

use axum::http::HeaderMap;
use domain::MerchantId;
use serde_json;

use crate::error::GatewayError;
use crate::persistence::PersistenceError;
use crate::routes::{CheckoutRequest, CheckoutResponse};
use crate::services::{process_checkout, CheckoutInput};
use crate::state::AppState;

/// Nombre del header HTTP exigido en checkout.
pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

const MAX_KEY_LEN: usize = 256;

/// Resultado cacheado de un checkout idempotente.
#[derive(Debug, Clone, PartialEq)]
pub enum CachedCheckoutResult {
    Success(CheckoutResponse),
    Error(GatewayError),
}

impl CachedCheckoutResult {
    pub(crate) fn into_result(self) -> Result<CheckoutResponse, GatewayError> {
        match self {
            Self::Success(response) => Ok(response),
            Self::Error(error) => Err(error),
        }
    }
}

fn persistence_to_gateway_error(error: PersistenceError) -> GatewayError {
    match error {
        PersistenceError::Internal(message) if message.contains("Idempotency-Key") => {
            GatewayError::Conflict(message)
        }
        PersistenceError::Internal(message) if message.contains("idempotente en curso") => {
            GatewayError::Conflict(message)
        }
        PersistenceError::Internal(message) if message.contains("en uso") => {
            GatewayError::Conflict(message)
        }
        other => GatewayError::Internal(other.to_string()),
    }
}

/// Extrae y valida el header `Idempotency-Key`.
pub fn extract_idempotency_key(headers: &HeaderMap) -> Result<String, GatewayError> {
    let value = headers
        .get(IDEMPOTENCY_KEY_HEADER)
        .ok_or(GatewayError::MissingIdempotencyKey)?
        .to_str()
        .map_err(|_| GatewayError::MissingIdempotencyKey)?
        .trim();

    if value.is_empty() {
        return Err(GatewayError::MissingIdempotencyKey);
    }

    if value.len() > MAX_KEY_LEN {
        return Err(GatewayError::InvalidRequest);
    }

    Ok(value.to_string())
}

/// Huella estable del payload de checkout para detectar reutilización conflictiva.
pub fn request_fingerprint(request: &CheckoutRequest) -> Result<String, GatewayError> {
    serde_json::to_string(request)
        .map_err(|err| GatewayError::Internal(format!("fingerprint checkout: {err}")))
}

/// Ejecuta checkout con idempotencia por comercio + clave (UC-01 / D9).
pub async fn process_checkout_idempotent(
    state: &AppState,
    merchant_id: MerchantId,
    idempotency_key: String,
    input: CheckoutInput,
) -> Result<CheckoutResponse, GatewayError> {
    let fingerprint = request_fingerprint(&input.request)?;

    if let Some(cached) = state
        .store()
        .idempotency_begin(merchant_id, &idempotency_key, &fingerprint)
        .await
        .map_err(persistence_to_gateway_error)?
    {
        return cached.into_result();
    }

    let result = process_checkout(state, input).await;
    state
        .store()
        .idempotency_complete(merchant_id, &idempotency_key, &fingerprint, &result)
        .await
        .map_err(persistence_to_gateway_error)?;

    result
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::http::HeaderMap;
    use domain::{FundingType, MerchantId};
    use oracle_client::{
        AuthorizeResponse, HealthResponse, OracleClient, OracleClientError, ReleaseHoldResponse,
    };
    use rail_switcher::RailSwitcher;
    use settlement_adapters::SettlementEngine;
    use uuid::Uuid;

    use super::*;
    use crate::config::AppConfig;
    use crate::routes::CheckoutCardPayload;
    use crate::services::rails::default_rail_configs;
    use crate::services::{MerchantRegistry, RailContext};
    use crate::state::AppState;

    struct CountingOracle {
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait]
    impl OracleClient for CountingOracle {
        async fn health(&self) -> Result<HealthResponse, OracleClientError> {
            Ok(HealthResponse {
                status: "ok".to_string(),
                service: "mock".to_string(),
            })
        }

        async fn authorize(
            &self,
            _request: oracle_client::AuthorizeRequest,
            _options: oracle_client::RequestOptions,
        ) -> Result<AuthorizeResponse, OracleClientError> {
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(AuthorizeResponse {
                hold_id: Uuid::new_v4(),
                brand: "visa".to_string(),
                brand_code: 1,
                last_four: "1111".to_string(),
                gateway_request_id: Uuid::new_v4(),
            })
        }

        async fn release_hold_request(
            &self,
            _request: oracle_client::ReleaseHoldRequest,
            _options: oracle_client::RequestOptions,
        ) -> Result<ReleaseHoldResponse, OracleClientError> {
            Ok(ReleaseHoldResponse {
                hold_id: Uuid::new_v4(),
                status: "released".to_string(),
            })
        }
    }

    fn sample_request() -> CheckoutRequest {
        CheckoutRequest {
            amount: 100.0,
            currency: "USD".to_string(),
            card: CheckoutCardPayload {
                pan: "4111111111111111".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "2030".to_string(),
                cvv: "123".to_string(),
                cardholder: "Test".to_string(),
            },
            funding_type: Some(FundingType::TraditionalBank),
        }
    }

    fn test_state(calls: Arc<std::sync::atomic::AtomicUsize>) -> (AppState, MerchantId) {
        let merchant_id = MerchantId::new(Uuid::new_v4());
        let config = Arc::new(AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            oracle_base_url: "http://mock".to_string(),
            oracle_api_key: "key".to_string(),
            oracle_timeout_secs: 2,
            oracle_health_check: false,
            default_merchant_id: merchant_id,
            merchant_api_keys: vec![],
            bootstrap_test_api_key: None,
            merchant_default_funding_type: None,
            rail_fallback_enabled: true,
            rail_configs: default_rail_configs(),
            database_url: None,
        });

        let state = AppState::from_parts(
            config,
            Arc::new(CountingOracle { calls }),
            SettlementEngine::with_stub_adapters(),
            RailSwitcher,
            RailContext::default(),
            Arc::new(MerchantRegistry::single("sk_test_validkey1", merchant_id)),
        );
        (state, merchant_id)
    }

    #[test]
    fn extract_idempotency_key_requires_header() {
        let headers = HeaderMap::new();
        assert!(matches!(
            extract_idempotency_key(&headers),
            Err(GatewayError::MissingIdempotencyKey)
        ));
    }

    #[test]
    fn extract_idempotency_key_rejects_empty_value() {
        let mut headers = HeaderMap::new();
        headers.insert(IDEMPOTENCY_KEY_HEADER, "   ".parse().expect("header"));
        assert!(matches!(
            extract_idempotency_key(&headers),
            Err(GatewayError::MissingIdempotencyKey)
        ));
    }

    #[tokio::test]
    async fn replays_success_without_second_oracle_call() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (state, merchant_id) = test_state(calls.clone());
        let input = CheckoutInput {
            merchant_id,
            request: sample_request(),
            caller_ip: Some("127.0.0.1".to_string()),
        };

        let first = process_checkout_idempotent(&state, merchant_id, "key-1".to_string(), input.clone())
            .await
            .expect("first");
        let second = process_checkout_idempotent(&state, merchant_id, "key-1".to_string(), input)
            .await
            .expect("second");

        assert_eq!(first.transaction_id, second.transaction_id);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rejects_same_key_with_different_payload() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (state, merchant_id) = test_state(calls);

        process_checkout_idempotent(
            &state,
            merchant_id,
            "key-2".to_string(),
            CheckoutInput {
                merchant_id,
                request: sample_request(),
                caller_ip: None,
            },
        )
        .await
        .expect("first");

        let mut different = sample_request();
        different.amount = 200.0;
        let err = process_checkout_idempotent(
            &state,
            merchant_id,
            "key-2".to_string(),
            CheckoutInput {
                merchant_id,
                request: different,
                caller_ip: None,
            },
        )
        .await
        .expect_err("conflict");

        assert!(matches!(err, GatewayError::Conflict(_)));
    }
}
