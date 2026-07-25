//! Trait de consulta de saldo por riel (UC-04).

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::funds::FundingType;

use super::error::RailAdapterError;

/// Proveedor de saldo bruto por riel antes de holds y spread buffer.
#[async_trait]
pub trait RailBalanceProvider: Send + Sync {
    /// Consulta el saldo disponible en el riel indicado.
    ///
    /// # Inputs
    /// - `funding_type`: riel a consultar.
    /// - `currency`: moneda ISO 4217 o símbolo del activo (p. ej. USDC).
    ///
    /// # Returns
    /// Saldo bruto reportado por el riel, sin descontar holds ni spread.
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError>;
}
