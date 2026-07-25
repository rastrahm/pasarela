//! Eventos de auditoría estructurados permitidos (sin PII).

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::funds::FundingType;
use crate::validation::{CardBrand, CardValidationResult};

/// Registra el inicio del flujo de autorización (post-Luhn).
pub fn authorization_started(
    gateway_request_id: Uuid,
    validation: &CardValidationResult,
    amount: Decimal,
    currency: &str,
    funding_type: FundingType,
    caller_ip: Option<&str>,
) {
    tracing::info!(
        event = "authorization_started",
        gateway_request_id = %gateway_request_id,
        brand = ?validation.brand,
        brand_code = validation.brand_code,
        last_four = %validation.last_four,
        token_hash = %validation.token_hash,
        amount = %amount,
        currency = %currency,
        funding_type = ?funding_type,
        caller_ip = caller_ip.unwrap_or("unknown"),
        "inicio de autorización"
    );
}

/// Registra autorización aprobada y hold creado.
pub fn authorization_approved(
    gateway_request_id: Uuid,
    hold_id: Uuid,
    brand: CardBrand,
    brand_code: u8,
    last_four: &str,
    amount: Decimal,
    currency: &str,
    funding_type: FundingType,
) {
    tracing::info!(
        event = "authorization_approved",
        gateway_request_id = %gateway_request_id,
        hold_id = %hold_id,
        brand = ?brand,
        brand_code = brand_code,
        last_four = %last_four,
        amount = %amount,
        currency = %currency,
        funding_type = ?funding_type,
        "autorización aprobada — hold creado"
    );
}

/// Registra rechazo de autorización con motivo seguro.
pub fn authorization_rejected(
    gateway_request_id: Uuid,
    reason: &str,
    brand: CardBrand,
    last_four: &str,
    amount: Decimal,
    funding_type: FundingType,
) {
    tracing::warn!(
        event = "authorization_rejected",
        gateway_request_id = %gateway_request_id,
        reason = reason,
        brand = ?brand,
        last_four = %last_four,
        amount = %amount,
        funding_type = ?funding_type,
        "autorización rechazada"
    );
}

/// Registra liberación de hold.
pub fn hold_released(hold_id: Uuid, status: &str) {
    tracing::info!(
        event = "hold_released",
        hold_id = %hold_id,
        status = status,
        "hold liberado"
    );
}

/// Registra tarjeta inválida sin exponer el PAN.
pub fn invalid_card_attempt(gateway_request_id: Uuid) {
    tracing::warn!(
        event = "invalid_card",
        gateway_request_id = %gateway_request_id,
        "tarjeta inválida — Luhn o marca desconocida"
    );
}

/// Registra denegación de acceso del Gateway (UC-11).
pub fn gateway_access_denied(reason: &str, client_ip: &str) {
    tracing::warn!(
        event = "gateway_access_denied",
        reason = reason,
        client_ip = %client_ip,
        "acceso interno denegado"
    );
}

/// Registra rate limit excedido.
pub fn rate_limit_exceeded(client_ip: &str) {
    tracing::warn!(
        event = "rate_limit_exceeded",
        client_ip = %client_ip,
        "rate limit excedido"
    );
}

/// Registra fallo de antifraude o riel (fail closed).
pub fn dependency_unavailable(dependency: &str, reason: &str, gateway_request_id: Option<Uuid>) {
    match gateway_request_id {
        Some(id) => tracing::warn!(
            event = "dependency_unavailable",
            dependency = dependency,
            reason = %reason,
            gateway_request_id = %id,
            "dependencia no disponible — fail closed"
        ),
        None => tracing::warn!(
            event = "dependency_unavailable",
            dependency = dependency,
            reason = %reason,
            "dependencia no disponible — fail closed"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::capture::run_with_log_capture;
    use crate::logging::redact::contains_forbidden_pii;
    use std::str::FromStr;

    fn sample_validation() -> CardValidationResult {
        CardValidationResult {
            valid: true,
            brand: CardBrand::Visa,
            brand_code: 1,
            last_four: "1111".to_string(),
            token_hash: "tok_abc123".to_string(),
        }
    }

    #[test]
    fn authorization_events_do_not_contain_pan() {
        let pan = "4111111111111111";
        let validation = sample_validation();

        let (_, logs) = run_with_log_capture(|| {
            authorization_started(
                Uuid::new_v4(),
                &validation,
                Decimal::from_str("100").expect("decimal"),
                "USD",
                FundingType::TraditionalBank,
                Some("127.0.0.1"),
            );
            authorization_approved(
                Uuid::new_v4(),
                Uuid::new_v4(),
                CardBrand::Visa,
                1,
                "1111",
                Decimal::from_str("100").expect("decimal"),
                "USD",
                FundingType::TraditionalBank,
            );
        });

        assert!(!logs.contains(pan));
        assert!(!contains_forbidden_pii(&logs));
        assert!(logs.contains("authorization_started"));
        assert!(logs.contains("last_four"));
    }
}
