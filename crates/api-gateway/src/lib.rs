//! API Gateway — orquestador de checkout multi-rail.

pub mod config;
pub mod error;
pub mod logging;
pub mod persistence;
pub mod routes;
pub mod services;
pub mod state;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderName, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::logging::http_trace_layer;
use crate::state::AppState;

fn cors_layer() -> CorsLayer {
    let mut origins = vec![
        "http://127.0.0.1:5173"
            .parse()
            .expect("origin dev 127.0.0.1"),
        "http://localhost:5173"
            .parse()
            .expect("origin dev localhost"),
    ];

    if let Ok(raw) = std::env::var("GATEWAY_CORS_ORIGINS") {
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(origin) = trimmed.parse() {
                origins.push(origin);
            } else {
                tracing::warn!(origin = trimmed, "GATEWAY_CORS_ORIGINS: origen inválido, omitido");
            }
        }
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("idempotency-key"),
        ])
}

/// Construye el router HTTP con rutas públicas y API v1.
///
/// # Inputs
/// - `state`: configuración, cliente Oracle y motor de liquidación.
///
/// # Returns
/// Router Axum listo para `axum::serve`.
pub fn build_app(state: AppState) -> Router {
    routes::create_router(Arc::new(state))
        .layer(cors_layer())
        .layer(http_trace_layer())
}
