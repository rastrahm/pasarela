//! Modelos persistidos del Gateway.

use domain::{FundingType, MerchantId, TransactionStatus};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Estado de un settlement persistido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementStatus {
    Completed,
    Failed,
}

/// Registro de liquidación en un riel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub rail_type: FundingType,
    pub proof: String,
    pub status: SettlementStatus,
}

/// Registro completo de una transacción en el Gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionRecord {
    pub transaction_id: Uuid,
    pub merchant_id: MerchantId,
    pub status: TransactionStatus,
    pub amount: Decimal,
    pub currency: String,
    pub rail_used: Option<FundingType>,
    pub settlement_proof: Option<String>,
    pub oracle_hold_id: Option<Uuid>,
}

/// Evento de auditoría sin PII.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub transaction_id: Option<Uuid>,
    pub event_type: String,
    pub detail: String,
}
