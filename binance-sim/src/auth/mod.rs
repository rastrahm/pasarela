//! Middleware de autenticación para el simulador Binance.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::config::AppConfig;
use crate::error::AppError;

const API_KEY_HEADER: &str = "x-api-key";

/// Valida `X-API-KEY` en rutas internas (fail closed).
pub async fn require_api_key(
    State(config): State<Arc<AppConfig>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let provided = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if provided.is_empty() || provided != config.api_key {
        return Err(AppError::Unauthorized);
    }

    Ok(next.run(request).await)
}
