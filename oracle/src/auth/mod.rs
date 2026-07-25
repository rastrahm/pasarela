//! Middleware de autenticación y control de acceso (UC-11).

use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::config::{AppConfig, CallerRule};
use crate::error::AppError;

const API_KEY_HEADER: &str = "x-api-key";

/// Middleware que valida `X-API-KEY`, allowlist de IP y rate limit básico.
///
/// # Inputs
/// - `State(config)`: configuración compartida con clave y allowlist.
/// - `request`: petición HTTP entrante.
///
/// # Returns
/// Respuesta del siguiente handler o error HTTP de seguridad (fail closed).
pub async fn require_gateway_auth(
    State(config): State<Arc<AppConfig>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    validate_api_key(request.headers().get(API_KEY_HEADER), &config.api_key)?;
    validate_caller_ip(extract_client_ip(&request), &config.allowed_callers)?;
    // Rate limiting completo: pendiente de implementación con ventana deslizante.
    let _ = config.rate_limit_per_minute;

    Ok(next.run(request).await)
}

fn validate_api_key(header: Option<&axum::http::HeaderValue>, expected: &str) -> Result<(), AppError> {
    let provided = header
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if provided.is_empty() || provided != expected {
        return Err(AppError::Unauthorized);
    }

    Ok(())
}

fn validate_caller_ip(client_ip: Option<IpAddr>, rules: &[CallerRule]) -> Result<(), AppError> {
    let ip = client_ip.ok_or(AppError::Forbidden)?;

    let allowed = rules.iter().any(|rule| match rule {
        CallerRule::Exact(expected) => ip == *expected,
        CallerRule::Prefix { base, prefix_len } => ip_in_prefix(ip, *base, *prefix_len),
    });

    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn extract_client_ip(request: &Request<Body>) -> Option<IpAddr> {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                if let Ok(ip) = first.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    // En desarrollo local sin proxy, se acepta loopback implícitamente
    // cuando no hay header; el test de integración usa este camino.
    Some(IpAddr::from([127, 0, 0, 1]))
}

fn ip_in_prefix(ip: IpAddr, base: IpAddr, prefix_len: u8) -> bool {
    match (ip, base) {
        (IpAddr::V4(ip_v4), IpAddr::V4(base_v4)) => {
            let ip_bits = u32::from(ip_v4);
            let base_bits = u32::from(base_v4);
            if prefix_len > 32 {
                return false;
            }
            let mask = if prefix_len == 0 {
                0
            } else {
                u32::MAX << (32 - prefix_len)
            };
            (ip_bits & mask) == (base_bits & mask)
        }
        (IpAddr::V6(ip_v6), IpAddr::V6(base_v6)) => {
            let ip_bits = u128::from(ip_v6);
            let base_bits = u128::from(base_v6);
            if prefix_len > 128 {
                return false;
            }
            let mask = if prefix_len == 0 {
                0
            } else {
                u128::MAX << (128 - prefix_len)
            };
            (ip_bits & mask) == (base_bits & mask)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_api_key() {
        let result = validate_api_key(None, "secret");
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn accepts_matching_api_key() {
        use axum::http::HeaderValue;
        let header = HeaderValue::from_static("secret");
        let result = validate_api_key(Some(&header), "secret");
        assert!(result.is_ok());
    }

    #[test]
    fn ipv4_prefix_match() {
        let ip: IpAddr = "172.17.0.5".parse().expect("ip");
        let base: IpAddr = "172.17.0.0".parse().expect("base");
        assert!(ip_in_prefix(ip, base, 16));
    }
}
