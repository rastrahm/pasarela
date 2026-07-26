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
use tower_http::cors::CorsLayer;

use crate::logging::http_trace_layer;
use crate::state::AppState;

fn dev_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "http://127.0.0.1:5173".parse().expect("origin"),
            "http://localhost:5173".parse().expect("origin"),
        ])
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
        .layer(dev_cors_layer())
        .layer(http_trace_layer())
}
