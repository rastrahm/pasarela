//! Utilidades para detectar y evitar PII en logs.

use std::fmt;

/// Campos de tarjeta que nunca deben aparecer en logs estructurados.
pub const FORBIDDEN_LOG_FIELDS: &[&str] = &["pan", "cvv", "cardholder"];

/// Indica si un texto contiene datos prohibidos (PAN completo, CVV o nombre).
pub fn contains_forbidden_pii(text: &str) -> bool {
    contains_pan_like_sequence(text)
        || contains_cvv_marker(text)
        || contains_cardholder_marker(text)
}

/// Vista redactada de una tarjeta para depuración segura.
pub struct RedactedCard<'a> {
    pub expiry_month: &'a str,
    pub expiry_year: &'a str,
    pub last_four: Option<&'a str>,
}

impl fmt::Debug for RedactedCard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Card")
            .field("pan", &"[REDACTED]")
            .field("expiry_month", &self.expiry_month)
            .field("expiry_year", &self.expiry_year)
            .field("cvv", &"[REDACTED]")
            .field("cardholder", &"[REDACTED]")
            .field("last_four", &self.last_four.unwrap_or("****"))
            .finish()
    }
}

fn contains_pan_like_sequence(text: &str) -> bool {
    extract_digit_segments(text)
        .into_iter()
        .any(|segment| is_probable_pan(&segment))
}

fn extract_digit_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn is_probable_pan(digits: &str) -> bool {
    (13..=19).contains(&digits.len()) && luhn_valid(digits) && has_card_iin_prefix(digits)
}

fn has_card_iin_prefix(digits: &str) -> bool {
    if digits.starts_with('4') {
        return true;
    }

    if digits.starts_with("34") || digits.starts_with("37") {
        return true;
    }

    if digits.len() >= 2 {
        if let Ok(prefix) = digits[..2].parse::<u16>() {
            if (51..=55).contains(&prefix) {
                return true;
            }
        }
    }

    false
}

fn contains_cvv_marker(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("cvv") && text.chars().any(|c| c.is_ascii_digit())
}

fn contains_cardholder_marker(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("cardholder") && !lower.contains("[redacted]")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_valid_pan_in_text() {
        assert!(contains_forbidden_pii(
            "error processing 4111111111111111 for user"
        ));
    }

    #[test]
    fn ignores_last_four_only() {
        assert!(!contains_forbidden_pii("last_four=1111 brand=visa"));
    }

    #[test]
    fn detects_cvv_leak_pattern() {
        assert!(contains_forbidden_pii(r#"{"cvv":"123"}"#));
    }

    #[test]
    fn redacted_card_debug_hides_pan() {
        let card = RedactedCard {
            expiry_month: "12",
            expiry_year: "30",
            last_four: Some("1111"),
        };
        let rendered = format!("{card:?}");
        assert!(!rendered.contains("4111111111111111"));
        assert!(rendered.contains("[REDACTED]"));
    }
}
