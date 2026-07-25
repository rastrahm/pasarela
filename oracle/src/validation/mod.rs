//! Validación de tarjetas: Luhn, detección de marca y tokenización en memoria.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Payload de tarjeta recibido del Gateway (contrato compartido en `oracle-client`).
pub use oracle_client::CardPayload;

/// Resultado de validación sin exponer el PAN completo.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CardValidationResult {
    pub valid: bool,
    pub brand: CardBrand,
    pub brand_code: u8,
    pub last_four: String,
    pub token_hash: String,
}

/// Marcas de tarjeta soportadas.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    sqlx::Type,
)]
#[sqlx(type_name = "card_brand", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Unknown,
}

impl CardBrand {
    fn code(self) -> u8 {
        match self {
            Self::Visa => 1,
            Self::Mastercard => 2,
            Self::Amex => 3,
            Self::Unknown => 0,
        }
    }
}

/// Valida el PAN con Luhn, detecta marca y genera token hash en memoria.
///
/// # Inputs
/// - `payload`: datos de tarjeta; el PAN se descarta tras tokenizar.
///
/// # Returns
/// Resultado de validación o `InvalidCard` / error de marca desconocida.
pub fn validate_card(payload: &CardPayload) -> Result<CardValidationResult, AppError> {
    let digits: String = payload.pan.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 13 || !luhn_valid(&digits) {
        return Err(AppError::InvalidCard);
    }

    let brand = detect_brand(&digits);
    if brand == CardBrand::Unknown {
        return Err(AppError::InvalidCard);
    }

    let last_four = digits
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    let token_hash = hash_pan(&digits);

    Ok(CardValidationResult {
        valid: true,
        brand,
        brand_code: brand.code(),
        last_four,
        token_hash,
    })
}

fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut alternate = false;

    for ch in digits.chars().rev() {
        let Some(mut digit) = ch.to_digit(10) else {
            return false;
        };

        if alternate {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }

        sum += digit;
        alternate = !alternate;
    }

    sum % 10 == 0
}

fn detect_brand(digits: &str) -> CardBrand {
    if digits.starts_with('4') {
        return CardBrand::Visa;
    }

    if digits.len() >= 2 {
        if let Ok(prefix2) = digits[..2].parse::<u16>() {
            if (51..=55).contains(&prefix2) {
                return CardBrand::Mastercard;
            }
        }
        if digits.starts_with("34") || digits.starts_with("37") {
            return CardBrand::Amex;
        }
    }

    CardBrand::Unknown
}

fn hash_pan(digits: &str) -> String {
    // Hash determinístico simple para demo; reemplazar por HMAC-SHA256 en producción.
    format!("tok_{:x}", simple_hash(digits))
}

fn simple_hash(input: &str) -> u64 {
    input.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload(pan: &str) -> CardPayload {
        CardPayload {
            pan: pan.to_string(),
            expiry_month: "12".to_string(),
            expiry_year: "30".to_string(),
            cvv: "123".to_string(),
            cardholder: "Demo User".to_string(),
        }
    }

    #[test]
    fn valid_visa_passes_luhn() {
        let result = validate_card(&sample_payload("4111111111111111"));
        assert!(result.is_ok());
        assert_eq!(result.expect("ok").brand, CardBrand::Visa);
    }

    #[test]
    fn invalid_pan_fails_luhn() {
        let result = validate_card(&sample_payload("4111111111111112"));
        assert!(matches!(result, Err(AppError::InvalidCard)));
    }

    #[test]
    fn unknown_brand_rejected() {
        let result = validate_card(&sample_payload("6011111111111117"));
        assert!(matches!(result, Err(AppError::InvalidCard)));
    }
}
