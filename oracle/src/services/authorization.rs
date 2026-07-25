//! Orquestación de autorización con transacción única.

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::antifraud_client::{AntifraudClientError, ScoreRequest};
use crate::error::AppError;
use crate::funds::{evaluate_funds, FundingType};
use crate::persistence::error::StoreError;
use crate::persistence::models::{
    AuditLogEntry, AuthResult, CreateAuthorizationRequest, CreateHold, HoldStatus,
};
use crate::persistence::AppState;
use crate::logging::{
    authorization_approved, authorization_rejected, authorization_started,
    dependency_unavailable, hold_released, invalid_card_attempt,
};
use crate::validation::{validate_card, CardPayload, CardValidationResult};

/// Solicitud de autorización recibida del Gateway.
#[derive(Clone)]
pub struct AuthorizeInput {
    pub gateway_request_id: Uuid,
    pub card: CardPayload,
    pub amount: Decimal,
    pub currency: String,
    pub funding_type: FundingType,
    pub caller_ip: Option<String>,
}

/// Respuesta de autorización exitosa.
#[derive(Debug, Clone)]
pub struct AuthorizeOutput {
    pub hold_id: Uuid,
    pub brand: String,
    pub brand_code: u8,
    pub last_four: String,
    pub gateway_request_id: Uuid,
}

/// Resultado de liberación de hold.
#[derive(Debug, Clone)]
pub struct ReleaseHoldOutput {
    pub hold_id: Uuid,
    pub status: HoldStatus,
}

/// Autoriza tarjeta, evalúa antifraude, fondos y persiste hold (UC-03 + UC-12 + UC-04).
pub async fn authorize(
    state: &AppState,
    input: AuthorizeInput,
) -> Result<AuthorizeOutput, AppError> {
    let validation = match validate_card(&input.card) {
        Ok(result) => result,
        Err(AppError::InvalidCard) => {
            invalid_card_attempt(input.gateway_request_id);
            return Err(AppError::InvalidCard);
        }
        Err(err) => return Err(err),
    };

    authorization_started(
        input.gateway_request_id,
        &validation,
        input.amount,
        &input.currency,
        input.funding_type,
        input.caller_ip.as_deref(),
    );

    score_antifraud(state, &input, &validation).await?;

    let fund_status = match evaluate_funds(
        state.rail_provider.as_ref(),
        &state.config,
        state.hold_store.as_ref(),
        input.amount,
        &input.currency,
        input.funding_type,
    )
    .await
    {
        Ok(status) => status,
        Err(err) => {
            persist_rejected_request(state, &input, &validation, "rail_unavailable").await?;
            dependency_unavailable(
                "rail",
                &err.to_string(),
                Some(input.gateway_request_id),
            );
            return Err(AppError::RailUnavailable);
        }
    };

    if !fund_status.sufficient {
        persist_rejected_request(state, &input, &validation, "insufficient_funds").await?;
        authorization_rejected(
            input.gateway_request_id,
            "insufficient_funds",
            validation.brand,
            &validation.last_four,
            input.amount,
            input.funding_type,
        );
        return Err(AppError::InsufficientFunds);
    }

    let auth_request_id = Uuid::new_v4();
    let hold_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::seconds(state.config.hold_ttl_secs as i64);

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let auth_record = state
        .authorization_store
        .create_request(
            &mut tx,
            CreateAuthorizationRequest {
                id: auth_request_id,
                gateway_request_id: input.gateway_request_id,
                funding_type: input.funding_type,
                amount: input.amount,
                currency: input.currency.clone(),
                brand: validation.brand,
                brand_code: validation.brand_code as i16,
                card_token_hash: validation.token_hash.clone(),
                result: AuthResult::Approved,
            },
        )
        .await
        .map_err(map_store_error)?;

    let hold_record = state
        .hold_store
        .create_hold(
            &mut tx,
            CreateHold {
                id: hold_id,
                authorization_request_id: auth_record.id,
                funding_type: input.funding_type,
                amount: input.amount,
                currency: input.currency.clone(),
                expires_at,
            },
        )
        .await
        .map_err(map_store_error)?;

    state
        .authorization_store
        .append_audit_log(
            &mut tx,
            AuditLogEntry {
                id: Uuid::new_v4(),
                hold_id: Some(hold_record.id),
                authorization_request_id: Some(auth_record.id),
                event_type: "hold_created".to_string(),
                detail: format!(
                    "hold={} riel={:?} amount={} currency={}",
                    hold_record.id, input.funding_type, input.amount, input.currency
                ),
                caller_ip: input.caller_ip.clone(),
            },
        )
        .await
        .map_err(map_store_error)?;

    tx.commit()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    authorization_approved(
        input.gateway_request_id,
        hold_record.id,
        validation.brand,
        validation.brand_code,
        &validation.last_four,
        input.amount,
        &input.currency,
        input.funding_type,
    );

    Ok(AuthorizeOutput {
        hold_id: hold_record.id,
        brand: format!("{:?}", validation.brand).to_lowercase(),
        brand_code: validation.brand_code,
        last_four: validation.last_four,
        gateway_request_id: input.gateway_request_id,
    })
}

/// Libera un hold activo; idempotente para Released/Expired (Decisión 7).
pub async fn release_hold(state: &AppState, hold_id: Uuid) -> Result<ReleaseHoldOutput, AppError> {
    let hold = state
        .hold_store
        .release_hold(hold_id)
        .await
        .map_err(map_store_error)?;

    if hold.status == HoldStatus::Released {
        hold_released(hold.id, "released");
        let audit = AuditLogEntry {
            id: Uuid::new_v4(),
            hold_id: Some(hold.id),
            authorization_request_id: Some(hold.authorization_request_id),
            event_type: "hold_released".to_string(),
            detail: format!("hold={} status=released", hold.id),
            caller_ip: None,
        };
        state
            .authorization_store
            .append_audit_log_standalone(audit)
            .await
            .map_err(map_store_error)?;
    }

    Ok(ReleaseHoldOutput {
        hold_id: hold.id,
        status: hold.status,
    })
}

async fn score_antifraud(
    state: &AppState,
    input: &AuthorizeInput,
    validation: &CardValidationResult,
) -> Result<(), AppError> {
    let request = ScoreRequest {
        gateway_request_id: input.gateway_request_id,
        amount: input.amount,
        currency: input.currency.clone(),
        funding_type: input.funding_type,
        token_hash: validation.token_hash.clone(),
    };

    match state.antifraud_client.score(request).await {
        Ok(_) => Ok(()),
        Err(AntifraudClientError::Declined) => {
            persist_rejected_request(state, input, validation, "fraud_declined").await?;
            authorization_rejected(
                input.gateway_request_id,
                "fraud_declined",
                validation.brand,
                &validation.last_four,
                input.amount,
                input.funding_type,
            );
            Err(AppError::FraudDeclined)
        }
        Err(AntifraudClientError::Unavailable(reason)) => {
            persist_rejected_request(state, input, validation, "antifraud_unavailable").await?;
            dependency_unavailable(
                "antifraud",
                &reason,
                Some(input.gateway_request_id),
            );
            Err(AppError::RailUnavailable)
        }
        Err(AntifraudClientError::InvalidResponse(reason)) => {
            persist_rejected_request(state, input, validation, "antifraud_invalid_response").await?;
            dependency_unavailable(
                "antifraud",
                &reason,
                Some(input.gateway_request_id),
            );
            Err(AppError::RailUnavailable)
        }
    }
}

async fn persist_rejected_request(
    state: &AppState,
    input: &AuthorizeInput,
    validation: &CardValidationResult,
    reason: &str,
) -> Result<(), AppError> {
    let auth_request_id = Uuid::new_v4();
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    state
        .authorization_store
        .create_request(
            &mut tx,
            CreateAuthorizationRequest {
                id: auth_request_id,
                gateway_request_id: input.gateway_request_id,
                funding_type: input.funding_type,
                amount: input.amount,
                currency: input.currency.clone(),
                brand: validation.brand,
                brand_code: validation.brand_code as i16,
                card_token_hash: validation.token_hash.clone(),
                result: AuthResult::Rejected,
            },
        )
        .await
        .map_err(map_store_error)?;

    state
        .authorization_store
        .append_audit_log(
            &mut tx,
            AuditLogEntry {
                id: Uuid::new_v4(),
                hold_id: None,
                authorization_request_id: Some(auth_request_id),
                event_type: "authorization_rejected".to_string(),
                detail: format!(
                    "reason={reason} riel={:?} amount={}",
                    input.funding_type, input.amount
                ),
                caller_ip: input.caller_ip.clone(),
            },
        )
        .await
        .map_err(map_store_error)?;

    tx.commit()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    Ok(())
}

fn map_store_error(err: StoreError) -> AppError {
    match err {
        StoreError::NotFound => AppError::NotFound,
        StoreError::InvalidHoldState(status) => AppError::Conflict(status),
        StoreError::Database(db_err) => AppError::Internal(db_err.to_string()),
    }
}

/// Convierte un monto f64 del JSON a Decimal de forma segura.
pub fn decimal_from_f64(value: f64) -> Result<Decimal, AppError> {
    Decimal::from_str(&value.to_string()).map_err(|_| AppError::InvalidCard)
}
