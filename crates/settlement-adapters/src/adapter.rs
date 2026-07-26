//! Trait Strategy para adaptadores de liquidación por riel.

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::context::SettlementContext;

/// Adaptador de liquidación para un riel concreto (Strategy pattern).
///
/// Cada implementación encapsula la lógica de asentamiento de un proveedor:
/// banco tradicional, Binance CEX o Solana on-chain.
#[async_trait]
pub trait SettlementAdapter: Send + Sync {
    /// Riel que implementa este adaptador.
    fn rail(&self) -> FundingType;

    /// Ejecuta la liquidación consumiendo el hold del Oracle.
    ///
    /// # Inputs
    /// - `context`: hold, monto, comercio y metadatos de la transacción.
    ///
    /// # Returns
    /// [`SettlementReceipt`] con la prueba del riel o [`LiquidityError`] si falla.
    async fn settle(&self, context: SettlementContext) -> Result<SettlementReceipt, LiquidityError>;
}
