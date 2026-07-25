//! Configuración de rieles y preferencias del comercio.

use domain::{Amount, Currency, FundingType};
use rust_decimal::Decimal;

/// Configuración operativa de un riel (`RAIL_CONFIG` en el ER).
#[derive(Debug, Clone, PartialEq)]
pub struct RailConfig {
    pub funding_type: FundingType,
    pub enabled: bool,
    /// Prioridad ascendente: menor valor = mayor preferencia en fallback.
    pub priority: u32,
    pub fee_percent: Decimal,
    pub spread_buffer: Decimal,
}

impl RailConfig {
    /// Costo efectivo estimado como porcentaje del monto (fee + spread buffer).
    pub fn effective_cost_percent(&self) -> Decimal {
        self.fee_percent + self.spread_buffer
    }
}

/// Preferencia de riel del usuario o comercio (`RAIL_PREFERENCE`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RailPreference {
    /// Riel elegido explícitamente en checkout (UC-02).
    pub preferred_rail: Option<FundingType>,
    /// Riel por defecto del comercio si no hay preferencia explícita.
    pub merchant_default: Option<FundingType>,
    /// Permite fallback automático (UC-08 / D3).
    pub fallback_enabled: bool,
}

/// Snapshot de disponibilidad de un riel consultado vía Oracle o adaptador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailAvailability {
    pub rail: FundingType,
    pub operational: bool,
    pub funds_sufficient: bool,
}

impl RailAvailability {
    /// Indica si el riel puede usarse para el monto solicitado.
    pub fn is_viable(&self) -> bool {
        self.operational && self.funds_sufficient
    }
}

/// Entrada completa para la selección de riel.
#[derive(Debug, Clone, PartialEq)]
pub struct RailSelectionInput<'a> {
    pub amount: Amount,
    pub currency: &'a Currency,
    pub preference: &'a RailPreference,
    pub configs: &'a [RailConfig],
    pub availability: &'a [RailAvailability],
}
