//! Tipos de fondeo, holds y liquidación.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipo de riel de fondeo / liquidación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundingType {
    TraditionalBank,
    BinanceCex,
    SolanaWallet,
}

/// Identificador de hold off-chain devuelto por el Oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HoldId(pub Uuid);

impl HoldId {
    /// Crea un identificador de hold a partir de un UUID existente.
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

/// Resultado de evaluación de fondos en un riel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundStatus {
    pub sufficient: bool,
    pub available_amount: Decimal,
    pub currency: String,
}

/// Comprobante de liquidación exitosa (referencia bancaria o firma on-chain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementReceipt {
    pub proof: String,
    pub rail: FundingType,
}
