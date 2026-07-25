//! Modelos de persistencia del Oracle alineados con el ER documentado.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::funds::FundingType;
use crate::validation::CardBrand;

/// Resultado de una solicitud de autorización.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_result", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthResult {
    Approved,
    Rejected,
}

/// Estado del ciclo de vida de un hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "hold_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum HoldStatus {
    Active,
    Consumed,
    Released,
    Expired,
}

/// Registro persistido de una solicitud de autorización (sin PAN).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorizationRequestRecord {
    pub id: Uuid,
    pub gateway_request_id: Uuid,
    pub funding_type: FundingType,
    pub amount: Decimal,
    pub currency: String,
    pub brand: CardBrand,
    pub brand_code: i16,
    pub card_token_hash: String,
    pub result: AuthResult,
    pub created_at: DateTime<Utc>,
}

/// Hold temporal sobre fondos del riel activo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct HoldRecord {
    pub id: Uuid,
    pub authorization_request_id: Uuid,
    pub funding_type: FundingType,
    pub amount: Decimal,
    pub currency: String,
    pub status: HoldStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Entrada de auditoría sin PII.
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub hold_id: Option<Uuid>,
    pub authorization_request_id: Option<Uuid>,
    pub event_type: String,
    pub detail: String,
    pub caller_ip: Option<String>,
}

/// Datos para crear una solicitud de autorización aprobada.
#[derive(Debug, Clone)]
pub struct CreateAuthorizationRequest {
    pub id: Uuid,
    pub gateway_request_id: Uuid,
    pub funding_type: FundingType,
    pub amount: Decimal,
    pub currency: String,
    pub brand: CardBrand,
    pub brand_code: i16,
    pub card_token_hash: String,
    pub result: AuthResult,
}

/// Datos para crear un hold activo.
#[derive(Debug, Clone)]
pub struct CreateHold {
    pub id: Uuid,
    pub authorization_request_id: Uuid,
    pub funding_type: FundingType,
    pub amount: Decimal,
    pub currency: String,
    pub expires_at: DateTime<Utc>,
}
