//! Solicitud y respuesta de pago.

use serde::{Deserialize, Serialize};

use crate::card::CardPayload;
use crate::funding::FundingType;
use crate::ids::{Amount, Currency, MerchantId, TransactionId};

/// Estado del ciclo de vida de una transacción en el Gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Authorized,
    Held,
    Settled,
    Failed,
    Reversed,
}

/// Solicitud de pago recibida en checkout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub transaction_id: TransactionId,
    pub merchant_id: MerchantId,
    pub amount: Amount,
    pub currency: Currency,
    pub card: CardPayload,
    pub funding_type: Option<FundingType>,
}

/// Respuesta de pago al comercio / frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub transaction_id: TransactionId,
    pub status: TransactionStatus,
    pub rail_used: FundingType,
    pub settlement_proof: Option<String>,
}
