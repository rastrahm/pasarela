//! Trait de persistencia para holds y expiración TTL.

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::error::StoreError;
use super::models::{CreateHold, HoldRecord, HoldStatus};
use crate::funds::FundingType;
use sqlx::PgTransaction;

/// Almacén de holds temporales sobre fondos por riel.
#[async_trait]
pub trait HoldStore: Send + Sync {
    /// Crea un hold activo dentro de una transacción.
    async fn create_hold(
        &self,
        tx: &mut PgTransaction<'_>,
        hold: CreateHold,
    ) -> Result<HoldRecord, StoreError>;

    /// Obtiene un hold por ID, aplicando expiración lazy si corresponde.
    async fn get_hold(&self, hold_id: Uuid) -> Result<Option<HoldRecord>, StoreError>;

    /// Libera un hold activo (idempotente para Released/Expired).
    async fn release_hold(&self, hold_id: Uuid) -> Result<HoldRecord, StoreError>;

    /// Suma el monto total de holds activos para un riel (reserva de fondos).
    async fn sum_active_holds(&self, funding_type: FundingType) -> Result<Decimal, StoreError>;

    /// Marca como expirados todos los holds activos cuyo TTL venció.
    async fn expire_stale_holds(&self) -> Result<u64, StoreError>;

    /// Actualiza el estado de un hold dentro de una transacción.
    async fn update_status_in_tx(
        &self,
        tx: &mut PgTransaction<'_>,
        hold_id: Uuid,
        status: HoldStatus,
    ) -> Result<HoldRecord, StoreError>;
}
