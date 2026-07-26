//! Orquestador de checkout — autorización Oracle + liquidación + liberación de hold.

use std::net::IpAddr;

use domain::{
    Amount, Currency, FundingType, HoldId, TransactionId, TransactionStatus,
};
use oracle_client::{
    AuthorizeRequest, CardPayload, FundingType as OracleFundingType, RequestOptions,
};
use rust_decimal::Decimal;
use settlement_adapters::SettlementContext;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::persistence::{AuditEvent, SettlementRecord, SettlementStatus, TransactionRecord};
use crate::routes::{CheckoutRequest, CheckoutResponse};
use crate::services::rails::{select_rail, settlement_rail_id};
use crate::state::AppState;

/// Parámetros de una solicitud de checkout entrante.
#[derive(Clone)]
pub struct CheckoutInput {
    pub merchant_id: domain::MerchantId,
    pub request: CheckoutRequest,
    pub caller_ip: Option<String>,
}

async fn persist_audit(
    state: &AppState,
    transaction_id: Uuid,
    event_type: &str,
    detail: impl Into<String>,
) -> Result<(), GatewayError> {
    state
        .store()
        .append_audit(&AuditEvent {
            transaction_id: Some(transaction_id),
            event_type: event_type.to_string(),
            detail: detail.into(),
        })
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))
}

/// Ejecuta el flujo completo UC-01: selección de riel → authorize → settle → respuesta.
pub async fn process_checkout(
    state: &AppState,
    input: CheckoutInput,
) -> Result<CheckoutResponse, GatewayError> {
    let (amount, currency) = parse_amount_and_currency(&input.request)?;
    let transaction_id = TransactionId::generate();
    let merchant_id = input.merchant_id;

    let rail = select_rail(
        &state.rail_switcher,
        &state.rail_context,
        &input.request,
        amount,
        &currency,
    )?;

    state
        .save_transaction(TransactionRecord {
            transaction_id: transaction_id.0,
            merchant_id,
            status: TransactionStatus::Pending,
            amount: amount.value(),
            currency: currency.as_str().to_string(),
            rail_used: Some(rail),
            settlement_proof: None,
            oracle_hold_id: None,
        })
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?;

    persist_audit(
        state,
        transaction_id.0,
        "checkout.started",
        format!("rail={rail:?}"),
    )
    .await?;

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
        .map_err(GatewayError::from_oracle_error)?;

    state
        .save_transaction(TransactionRecord {
            transaction_id: transaction_id.0,
            merchant_id,
            status: TransactionStatus::Held,
            amount: amount.value(),
            currency: currency.as_str().to_string(),
            rail_used: Some(rail),
            settlement_proof: None,
            oracle_hold_id: Some(auth.hold_id),
        })
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?;

    persist_audit(state, transaction_id.0, "checkout.authorized", "hold created").await?;

    let settlement_context = SettlementContext {
        hold_id: HoldId::new(auth.hold_id),
        transaction_id,
        merchant_id,
        amount,
        currency: currency.clone(),
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
                settlement_proof: Some(receipt.proof.clone()),
            };

            state
                .save_transaction(TransactionRecord {
                    transaction_id: transaction_id.0,
                    merchant_id,
                    status: TransactionStatus::Settled,
                    amount: amount.value(),
                    currency: currency.as_str().to_string(),
                    rail_used: Some(receipt.rail),
                    settlement_proof: response.settlement_proof.clone(),
                    oracle_hold_id: Some(auth.hold_id),
                })
                .await
                .map_err(|err| GatewayError::Internal(err.to_string()))?;

            state
                .store()
                .insert_settlement(&SettlementRecord {
                    id: Uuid::new_v4(),
                    transaction_id: transaction_id.0,
                    rail_type: receipt.rail,
                    proof: receipt.proof,
                    status: SettlementStatus::Completed,
                })
                .await
                .map_err(|err| GatewayError::Internal(err.to_string()))?;

            persist_audit(state, transaction_id.0, "checkout.settled", "settlement ok").await?;

            Ok(response)
        }
        Err(settlement_err) => {
            release_hold_on_settlement_failure(
                state,
                transaction_id.0,
                auth.hold_id,
                options,
            )
            .await;

            state
                .save_transaction(TransactionRecord {
                    transaction_id: transaction_id.0,
                    merchant_id,
                    status: TransactionStatus::Failed,
                    amount: amount.value(),
                    currency: currency.as_str().to_string(),
                    rail_used: Some(rail),
                    settlement_proof: None,
                    oracle_hold_id: Some(auth.hold_id),
                })
                .await
                .map_err(|err| GatewayError::Internal(err.to_string()))?;

            persist_audit(state, transaction_id.0, "checkout.failed", "settlement failed").await?;

            Err(GatewayError::from_liquidity_error(settlement_err))
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

/// Libera el hold en Oracle cuando el settlement falla (UC-04 / paso 4.13).
async fn release_hold_on_settlement_failure(
    state: &AppState,
    transaction_id: Uuid,
    hold_id: Uuid,
    options: RequestOptions,
) {
    match state.oracle_client.release_hold(hold_id, options).await {
        Ok(response) => {
            if let Err(err) = persist_audit(
                state,
                transaction_id,
                "checkout.hold_released",
                format!("hold_id={hold_id} status={}", response.status),
            )
            .await
            {
                tracing::warn!(%transaction_id, %hold_id, %err, "audit hold_released falló");
            }
            tracing::info!(%hold_id, %transaction_id, "hold liberado tras fallo de settlement");
        }
        Err(err) => {
            if let Err(audit_err) = persist_audit(
                state,
                transaction_id,
                "checkout.hold_release_failed",
                format!("hold_id={hold_id} error={err}"),
            )
            .await
            {
                tracing::warn!(%transaction_id, %hold_id, %audit_err, "audit hold_release_failed falló");
            }
            tracing::warn!(
                %hold_id,
                %transaction_id,
                %err,
                "no se pudo liberar hold tras fallo de settlement"
            );
        }
    }
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
    use domain::{FundingType, LiquidityError, MerchantId};
    use oracle_client::{
        AuthorizeResponse, HealthResponse, OracleClient, OracleClientError, ReleaseHoldResponse,
    };
    use rail_switcher::RailSwitcher;
    use settlement_adapters::{MockSettlementAdapter, SettlementAdapter, SettlementEngine};
    use uuid::Uuid;

    use super::*;
    use crate::config::AppConfig;
    use crate::routes::CheckoutCardPayload;
    use crate::services::rails::default_rail_configs;
    use crate::services::{MerchantRegistry, RailContext};
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

    fn test_state(oracle: Arc<dyn OracleClient>) -> (AppState, domain::MerchantId) {
        test_state_with_settlement(oracle, SettlementEngine::with_stub_adapters())
    }

    fn test_state_with_settlement(
        oracle: Arc<dyn OracleClient>,
        settlement_engine: SettlementEngine,
    ) -> (AppState, domain::MerchantId) {
        let merchant_id = MerchantId::new(Uuid::new_v4());
        let config = Arc::new(AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            oracle_base_url: "http://mock".to_string(),
            oracle_api_key: "key".to_string(),
            oracle_timeout_secs: 2,
            oracle_health_check: false,
            database_url: None,
            default_merchant_id: merchant_id,
            merchant_api_keys: vec![],
            bootstrap_test_api_key: None,
            merchant_default_funding_type: None,
            rail_fallback_enabled: true,
            rail_configs: default_rail_configs(),
        });
        let state = AppState::from_parts(
            config,
            oracle,
            settlement_engine,
            RailSwitcher,
            RailContext::default(),
            Arc::new(MerchantRegistry::single("sk_test_validkey1", merchant_id)),
        );
        (state, merchant_id)
    }

    #[tokio::test]
    async fn checkout_happy_path_returns_settled() {
        let release_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (state, merchant_id) = test_state(Arc::new(MockOracle {
            authorize_ok: true,
            release_called: release_flag.clone(),
        }));

        let response = process_checkout(
            &state,
            CheckoutInput {
                merchant_id,
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
        let (state, merchant_id) = test_state(Arc::new(MockOracle {
            authorize_ok: false,
            release_called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }));

        let err = process_checkout(
            &state,
            CheckoutInput {
                merchant_id,
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
        let (state, merchant_id) = test_state(Arc::new(MockOracle {
            authorize_ok: true,
            release_called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }));

        let mut request = sample_request();
        request.amount = 0.0;

        let err = process_checkout(
            &state,
            CheckoutInput {
                merchant_id,
                request,
                caller_ip: None,
            },
        )
        .await
        .expect_err("error");

        assert!(matches!(err, GatewayError::InvalidCard));
    }

    #[tokio::test]
    async fn checkout_releases_hold_when_settlement_fails() {
        let release_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let oracle = Arc::new(MockOracle {
            authorize_ok: true,
            release_called: release_flag.clone(),
        });
        let failing_engine = SettlementEngine::new([Arc::new(
            MockSettlementAdapter::new(FundingType::TraditionalBank, "unused")
                .with_error(LiquidityError::SettlementFailed),
        )
            as Arc<dyn SettlementAdapter>]);
        let (state, merchant_id) = test_state_with_settlement(oracle, failing_engine);

        let err = process_checkout(
            &state,
            CheckoutInput {
                merchant_id,
                request: sample_request(),
                caller_ip: None,
            },
        )
        .await
        .expect_err("settlement failure");

        assert!(matches!(err, GatewayError::Internal(_)));
        assert!(release_flag.load(std::sync::atomic::Ordering::SeqCst));
    }
}
