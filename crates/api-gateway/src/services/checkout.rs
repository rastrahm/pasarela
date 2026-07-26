//! Orquestador de checkout — autorización Oracle + liquidación + liberación de hold.

use std::net::IpAddr;

use domain::{
    Amount, Currency, FundingType, HoldId, LiquidityError, TransactionId, TransactionStatus,
};
use oracle_client::{
    AuthorizeRequest, CardPayload, FundingType as OracleFundingType, RequestOptions,
};
use rust_decimal::Decimal;
use settlement_adapters::SettlementContext;

use crate::error::GatewayError;
use crate::routes::{CheckoutRequest, CheckoutResponse};
use crate::services::rails::{select_rail, settlement_rail_id};
use crate::state::{AppState, TransactionRecord};

/// Parámetros de una solicitud de checkout entrante.
pub struct CheckoutInput {
    pub request: CheckoutRequest,
    pub caller_ip: Option<String>,
}

/// Ejecuta el flujo completo UC-01: selección de riel → authorize → settle → respuesta.
pub async fn process_checkout(
    state: &AppState,
    input: CheckoutInput,
) -> Result<CheckoutResponse, GatewayError> {
    let (amount, currency) = parse_amount_and_currency(&input.request)?;
    let transaction_id = TransactionId::generate();
    let merchant_id = state.default_merchant_id;

    let rail = select_rail(&state.rail_switcher, &input.request, amount, &currency)?;

    let gateway_request_id = transaction_id.0;
    let authorize_request = AuthorizeRequest {
        gateway_request_id,
        card: to_oracle_card(&input.request.card),
        amount: input.request.amount,
        currency: currency.as_str().to_string(),
        funding_type: to_oracle_funding_type(rail),
    };

    let options = RequestOptions {
        caller_ip: input.caller_ip.clone(),
    };

    let auth = state
        .oracle_client
        .authorize(authorize_request, options.clone())
        .await
        .map_err(map_oracle_error)?;

    let settlement_context = SettlementContext {
        hold_id: HoldId::new(auth.hold_id),
        transaction_id,
        merchant_id,
        amount,
        currency,
        brand_code: auth.brand_code,
        settlement_rail_id: settlement_rail_id(rail),
    };

    match state
        .settlement_engine
        .settle(rail, settlement_context)
        .await
    {
        Ok(receipt) => {
            let response = CheckoutResponse {
                transaction_id: transaction_id.0,
                status: TransactionStatus::Settled,
                rail_used: receipt.rail,
                settlement_proof: Some(receipt.proof),
            };
            state.store_transaction(TransactionRecord {
                transaction_id: transaction_id.0,
                status: TransactionStatus::Settled,
                rail_used: Some(receipt.rail),
                settlement_proof: response.settlement_proof.clone(),
            });
            Ok(response)
        }
        Err(settlement_err) => {
            let _ = state
                .oracle_client
                .release_hold(auth.hold_id, options)
                .await;

            state.store_transaction(TransactionRecord {
                transaction_id: transaction_id.0,
                status: TransactionStatus::Failed,
                rail_used: Some(rail),
                settlement_proof: None,
            });

            Err(map_liquidity_error(settlement_err))
        }
    }
}

fn parse_amount_and_currency(request: &CheckoutRequest) -> Result<(Amount, Currency), GatewayError> {
    if !request.amount.is_finite() || request.amount <= 0.0 {
        return Err(GatewayError::InvalidCard);
    }

    let decimal = Decimal::from_f64_retain(request.amount)
        .filter(|value| *value > Decimal::ZERO)
        .ok_or(GatewayError::InvalidCard)?;

    let amount = Amount::new(decimal);
    let currency = Currency::new(&request.currency).map_err(|_| GatewayError::InvalidCard)?;

    Ok((amount, currency))
}

fn to_oracle_card(card: &crate::routes::CheckoutCardPayload) -> CardPayload {
    CardPayload {
        pan: card.pan.clone(),
        expiry_month: card.expiry_month.clone(),
        expiry_year: card.expiry_year.clone(),
        cvv: card.cvv.clone(),
        cardholder: card.cardholder.clone(),
    }
}

fn to_oracle_funding_type(rail: FundingType) -> OracleFundingType {
    match rail {
        FundingType::TraditionalBank => OracleFundingType::TraditionalBank,
        FundingType::BinanceCex => OracleFundingType::BinanceCex,
        FundingType::SolanaWallet => OracleFundingType::SolanaWallet,
    }
}

fn map_oracle_error(error: oracle_client::OracleClientError) -> GatewayError {
    use oracle_client::OracleClientError;

    match error {
        OracleClientError::Unauthorized | OracleClientError::Forbidden => {
            GatewayError::Unauthorized
        }
        OracleClientError::InvalidCard => GatewayError::InvalidCard,
        OracleClientError::InsufficientFunds | OracleClientError::FraudDeclined => {
            GatewayError::InsufficientFunds
        }
        OracleClientError::RailUnavailable | OracleClientError::Unavailable(_) => {
            GatewayError::RailUnavailable
        }
        OracleClientError::Conflict => GatewayError::Conflict("conflicto en Oracle".to_string()),
        OracleClientError::TooManyRequests => {
            GatewayError::Internal("rate limit Oracle".to_string())
        }
        OracleClientError::NotFound => GatewayError::NotFound,
        OracleClientError::InternalError | OracleClientError::InvalidResponse(_) => {
            GatewayError::Internal(error.to_string())
        }
        OracleClientError::Api(body) => GatewayError::Internal(body.message),
    }
}

fn map_liquidity_error(error: LiquidityError) -> GatewayError {
    match error {
        LiquidityError::InsufficientFunds { .. } => GatewayError::InsufficientFunds,
        LiquidityError::RailUnavailable { .. } => GatewayError::RailUnavailable,
        LiquidityError::HoldNotFound
        | LiquidityError::HoldFailed
        | LiquidityError::SettlementFailed => GatewayError::Internal(error.to_string()),
    }
}

/// Extrae IP del caller para allowlist Oracle (`X-Forwarded-For` o localhost).
pub fn extract_caller_ip(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                return Some(first.trim().to_string());
            }
        }
    }

    Some(IpAddr::from([127, 0, 0, 1]).to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
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
    use crate::state::AppState;

    struct MockOracle {
        authorize_ok: bool,
        release_called: Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl OracleClient for MockOracle {
        async fn health(&self) -> Result<HealthResponse, OracleClientError> {
            Ok(HealthResponse {
                status: "ok".to_string(),
                service: "mock".to_string(),
            })
        }

        async fn authorize(
            &self,
            _request: AuthorizeRequest,
            _options: RequestOptions,
        ) -> Result<AuthorizeResponse, OracleClientError> {
            if self.authorize_ok {
                Ok(AuthorizeResponse {
                    hold_id: Uuid::new_v4(),
                    brand: "visa".to_string(),
                    brand_code: 1,
                    last_four: "1111".to_string(),
                    gateway_request_id: Uuid::new_v4(),
                })
            } else {
                Err(OracleClientError::InvalidCard)
            }
        }

        async fn release_hold_request(
            &self,
            _request: oracle_client::ReleaseHoldRequest,
            _options: RequestOptions,
        ) -> Result<ReleaseHoldResponse, OracleClientError> {
            self.release_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
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

    fn test_state(oracle: Arc<dyn OracleClient>) -> AppState {
        let config = Arc::new(AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            oracle_base_url: "http://mock".to_string(),
            oracle_api_key: "key".to_string(),
            oracle_timeout_secs: 2,
            database_url: None,
            default_merchant_id: MerchantId::new(Uuid::new_v4()),
        });
        AppState::from_parts(
            config,
            oracle,
            SettlementEngine::with_stub_adapters(),
            RailSwitcher,
        )
    }

    #[tokio::test]
    async fn checkout_happy_path_returns_settled() {
        let release_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let state = test_state(Arc::new(MockOracle {
            authorize_ok: true,
            release_called: release_flag.clone(),
        }));

        let response = process_checkout(
            &state,
            CheckoutInput {
                request: sample_request(),
                caller_ip: Some("127.0.0.1".to_string()),
            },
        )
        .await
        .expect("checkout");

        assert_eq!(response.status, TransactionStatus::Settled);
        assert_eq!(response.rail_used, FundingType::TraditionalBank);
        assert!(response.settlement_proof.is_some());
        assert!(!release_flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn checkout_maps_oracle_invalid_card_to_422() {
        let state = test_state(Arc::new(MockOracle {
            authorize_ok: false,
            release_called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }));

        let err = process_checkout(
            &state,
            CheckoutInput {
                request: sample_request(),
                caller_ip: None,
            },
        )
        .await
        .expect_err("error");

        assert!(matches!(err, GatewayError::InvalidCard));
    }

    #[tokio::test]
    async fn checkout_rejects_invalid_amount() {
        let state = test_state(Arc::new(MockOracle {
            authorize_ok: true,
            release_called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }));

        let mut request = sample_request();
        request.amount = 0.0;

        let err = process_checkout(
            &state,
            CheckoutInput {
                request,
                caller_ip: None,
            },
        )
        .await
        .expect_err("error");

        assert!(matches!(err, GatewayError::InvalidCard));
    }
}
