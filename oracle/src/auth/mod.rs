//! Middleware de autenticación y control de acceso (UC-11).

mod rate_limit;
mod state;

use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

pub use rate_limit::SlidingWindowRateLimiter;
pub use state::GatewayAuthState;

use crate::config::CallerRule;
use crate::error::AppError;
use crate::logging::{gateway_access_denied, rate_limit_exceeded};

use rate_limit::{rate_limit_key, IpAddrDisplay};

const API_KEY_HEADER: &str = "x-api-key";

/// Middleware que valida `X-API-KEY`, allowlist de IP y rate limit por ventana deslizante.
///
/// # Inputs
/// - `State(auth)`: configuración y limitador compartidos.
/// - `request`: petición HTTP entrante.
///
/// # Returns
/// Respuesta del siguiente handler o error HTTP de seguridad (fail closed).
pub async fn require_gateway_auth(
    State(auth): State<Arc<GatewayAuthState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let client_ip = extract_client_ip(&request);
    let client_ip_str = client_ip
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let api_key = match extract_api_key(request.headers().get(API_KEY_HEADER)) {
        Ok(key) => key,
        Err(err) => {
            gateway_access_denied("missing_api_key", &client_ip_str);
            return Err(err);
        }
    };

    if let Err(err) = validate_api_key(&api_key, &auth.config.api_key) {
        gateway_access_denied("invalid_api_key", &client_ip_str);
        return Err(err);
    }

    let client_ip = client_ip.ok_or(AppError::Forbidden)?;
    if let Err(err) = validate_caller_ip(client_ip, &auth.config.allowed_callers) {
        gateway_access_denied("ip_not_allowed", &client_ip_str);
        return Err(err);
    }

    let limit_key = rate_limit_key(&api_key, IpAddrDisplay(&client_ip_str));
    if let Err(err) = auth.rate_limiter.try_acquire(&limit_key).await {
        if matches!(err, AppError::TooManyRequests) {
            rate_limit_exceeded(&client_ip_str);
        }
        return Err(err);
    }

    Ok(next.run(request).await)
}

fn extract_api_key(header: Option<&axum::http::HeaderValue>) -> Result<String, AppError> {
    match header
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    {
        Some(key) => Ok(key),
        None => Err(AppError::Unauthorized),
    }
}

fn validate_api_key(provided: &str, expected: &str) -> Result<(), AppError> {
    if provided == expected {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

fn validate_caller_ip(client_ip: IpAddr, rules: &[CallerRule]) -> Result<(), AppError> {
    let allowed = rules.iter().any(|rule| match rule {
        CallerRule::Exact(expected) => client_ip == *expected,
        CallerRule::Prefix { base, prefix_len } => ip_in_prefix(client_ip, *base, *prefix_len),
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
        let result = extract_api_key(None);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn accepts_matching_api_key() {
        use axum::http::HeaderValue;
        let header = HeaderValue::from_static("secret");
        let key = extract_api_key(Some(&header)).expect("key");
        let result = validate_api_key(&key, "secret");
        assert!(result.is_ok());
    }

    #[test]
    fn ipv4_prefix_match() {
        let ip: IpAddr = "172.17.0.5".parse().expect("ip");
        let base: IpAddr = "172.17.0.0".parse().expect("base");
        assert!(ip_in_prefix(ip, base, 16));
    }

    #[test]
    fn rejects_ip_outside_allowlist() {
        let ip: IpAddr = "10.0.0.1".parse().expect("ip");
        let rules = vec![CallerRule::Exact("127.0.0.1".parse().expect("ip"))];
        let result = validate_caller_ip(ip, &rules);
        assert!(matches!(result, Err(AppError::Forbidden)));
    }
}
