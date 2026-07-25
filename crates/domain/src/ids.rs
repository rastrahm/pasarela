//! Newtypes de identidad y montos del dominio.

use std::fmt;
use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identificador único de una transacción en el Gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    /// Crea un identificador de transacción a partir de un UUID existente.
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// Genera un identificador aleatorio v4.
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identificador del comercio que origina el pago.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MerchantId(pub Uuid);

impl MerchantId {
    /// Crea un identificador de comercio a partir de un UUID existente.
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for MerchantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monto monetario con precisión decimal (sin unidad implícita).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Amount(Decimal);

impl Amount {
    /// Construye un monto a partir de un valor decimal.
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Construye un monto a partir de un entero (unidades principales).
    pub fn from_units(units: i64) -> Self {
        Self(Decimal::from(units))
    }

    /// Devuelve el valor decimal subyacente.
    pub fn value(self) -> Decimal {
        self.0
    }

    /// Indica si el monto es estrictamente positivo.
    pub fn is_positive(self) -> bool {
        self.0 > Decimal::ZERO
    }
}

impl From<Decimal> for Amount {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Código de moneda ISO 4217 (tres letras mayúsculas).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Currency(String);

impl Currency {
    /// Crea una moneda validando formato ISO 4217 (3 letras).
    pub fn new(code: impl Into<String>) -> Result<Self, CurrencyError> {
        let code = code.into();
        validate_currency_code(&code)?;
        Ok(Self(code.to_uppercase()))
    }

    /// Devuelve el código como slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Currency {
    type Err = CurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Currency::new(s)
    }
}

/// Error al construir un [`Currency`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CurrencyError {
    #[error("código de moneda inválido: debe ser ISO 4217 de 3 letras")]
    InvalidCode,
}

fn validate_currency_code(code: &str) -> Result<(), CurrencyError> {
    if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) {
        Ok(())
    } else {
        Err(CurrencyError::InvalidCode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_from_units_is_positive() {
        let amount = Amount::from_units(100);
        assert!(amount.is_positive());
    }

    #[test]
    fn currency_accepts_iso_code() {
        let currency = Currency::new("usd").expect("valid");
        assert_eq!(currency.as_str(), "USD");
    }

    #[test]
    fn currency_rejects_invalid_code() {
        assert_eq!(Currency::new("US"), Err(CurrencyError::InvalidCode));
    }
}
