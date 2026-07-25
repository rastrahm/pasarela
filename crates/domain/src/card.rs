//! Datos de tarjeta ficticia (PII en tránsito; tokenizar antes de persistir).

use std::fmt;

use serde::{Deserialize, Serialize};

/// PAN tokenizado o en tránsito — nunca persistir en claro.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardNumber(pub String);

impl CardNumber {
    /// Envuelve un número de tarjeta sin validar (validación en capa Oracle).
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Devuelve los últimos cuatro dígitos si el PAN tiene longitud suficiente.
    pub fn last_four(&self) -> Option<String> {
        if self.0.len() >= 4 {
            Some(self.0[self.0.len() - 4..].to_string())
        } else {
            None
        }
    }
}

impl fmt::Debug for CardNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardNumber")
            .field("pan", &"[REDACTED]")
            .finish()
    }
}

/// Payload de tarjeta enviado por el frontend vía Gateway.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CardPayload {
    pub pan: CardNumber,
    pub expiry_month: String,
    pub expiry_year: String,
    pub cvv: String,
    pub cardholder: String,
}

impl fmt::Debug for CardPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardPayload")
            .field("pan", &"[REDACTED]")
            .field("expiry_month", &self.expiry_month)
            .field("expiry_year", &self.expiry_year)
            .field("cvv", &"[REDACTED]")
            .field("cardholder", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_number_debug_redacts_pan() {
        let card = CardNumber::new("4111111111111111");
        let rendered = format!("{card:?}");
        assert!(!rendered.contains("4111111111111111"));
    }

    #[test]
    fn last_four_extracts_suffix() {
        let card = CardNumber::new("4111111111111111");
        assert_eq!(card.last_four(), Some("1111".to_string()));
    }
}
