//! Contexto de liquidación compartido por todos los adaptadores.

use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};

/// Datos necesarios para ejecutar `settle` en cualquier riel.
///
/// El Oracle ya creó el hold durante `/authorize`; el adaptador consume ese hold
/// y devuelve un [`domain::SettlementReceipt`] con la prueba específica del riel.
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementContext {
    /// Hold off-chain creado por el Oracle.
    pub hold_id: HoldId,
    /// Transacción del Gateway que origina la liquidación.
    pub transaction_id: TransactionId,
    /// Comercio beneficiario del pago.
    pub merchant_id: MerchantId,
    /// Monto autorizado a liquidar.
    pub amount: Amount,
    /// Moneda ISO 4217 de la transacción.
    pub currency: Currency,
    /// Código numérico de marca (Visa=1, MC=2, Amex=3) — sin PAN.
    pub brand_code: u8,
    /// Identificador del riel en `RAIL_CONFIG` (mapea a `settlement_rail_id` on-chain).
    pub settlement_rail_id: u64,
}
