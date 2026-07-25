//! Servicio Binance Spot simulado — biblioteca compartida.

pub mod auth;
pub mod config;
pub mod error;
pub mod routes;

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;

/// Construye el router HTTP del simulador Binance.
pub fn build_app(config: AppConfig) -> Router {
    routes::create_router(Arc::new(config))
}
