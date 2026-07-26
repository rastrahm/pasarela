//! Errores de dominio tipados con `thiserror`.

use thiserror::Error;

use crate::funding::FundingType;

/// Error al procesar un pago end-to-end.
#[derive(Debug, Error, PartialEq)]
pub enum PaymentError {
    #[error("tarjeta inválida")]
    InvalidCard,

    #[error("fondos insuficientes")]
    InsufficientFunds,

    #[error("transacción rechazada por antifraude")]
    FraudDeclined,

    #[error("ningún riel viable: {0}")]
    RailSelection(#[from] RailError),

    #[error("error de liquidez: {0}")]
    Liquidity(#[from] LiquidityError),

    #[error("error interno: {0}")]
    Internal(String),
}

/// Error al evaluar o mover fondos en un riel.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum LiquidityError {
    #[error("fondos insuficientes en riel {rail:?}")]
    InsufficientFunds { rail: FundingType },

    #[error("riel {rail:?} no disponible")]
    RailUnavailable { rail: FundingType },

    #[error("hold no encontrado")]
    HoldNotFound,

    #[error("fallo al crear hold")]
    HoldFailed,

    #[error("fallo en liquidación")]
    SettlementFailed,
}

/// Error al seleccionar un riel de fondeo.
#[derive(Debug, Error, PartialEq)]
pub enum RailError {
    #[error("ningún riel habilitado y viable")]
    NoRailAvailable,

    #[error("riel preferido {rail:?} no disponible y fallback deshabilitado")]
    PreferredUnavailableNoFallback { rail: FundingType },

    #[error("riel {rail:?} deshabilitado en configuración")]
    RailDisabled { rail: FundingType },

    #[error("riel {rail:?} sin fondos suficientes")]
    InsufficientFunds { rail: FundingType },

    #[error("riel {rail:?} no operativo")]
    Unavailable { rail: FundingType },
}
