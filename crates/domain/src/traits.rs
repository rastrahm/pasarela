//! Traits principales del dominio — procesamiento y liquidez.

use crate::error::{LiquidityError, PaymentError};
use crate::funding::{FundStatus, FundingType, HoldId, SettlementReceipt};
use crate::ids::Amount;
use crate::payment::{PaymentRequest, PaymentResponse};

/// Procesa una solicitud de pago end-to-end.
pub trait PaymentProcessor {
    /// Ejecuta autorización, hold y liquidación según el riel seleccionado.
    fn process(&self, request: PaymentRequest) -> Result<PaymentResponse, PaymentError>;
}

/// Evalúa liquidez y ejecuta hold/asentamiento en el riel activo.
pub trait LiquidityEngine {
    /// Consulta si hay fondos suficientes en el riel indicado.
    fn evaluate_funds(&self, amount: Amount, rail: FundingType) -> Result<FundStatus, LiquidityError>;

    /// Reserva fondos creando un hold off-chain.
    fn hold(&self, amount: Amount) -> Result<HoldId, LiquidityError>;

    /// Confirma la liquidación consumiendo un hold existente.
    fn settle(&self, hold_id: HoldId) -> Result<SettlementReceipt, LiquidityError>;
}
