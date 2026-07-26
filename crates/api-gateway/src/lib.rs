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

use crate::logging::http_trace_layer;
use crate::state::AppState;

/// Construye el router HTTP con rutas públicas y API v1.
///
/// # Inputs
/// - `state`: configuración, cliente Oracle y motor de liquidación.
///
/// # Returns
/// Router Axum listo para `axum::serve`.
pub fn build_app(state: AppState) -> Router {
    routes::create_router(Arc::new(state)).layer(http_trace_layer())
}
