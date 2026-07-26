//! Servicio Binance Spot simulado — biblioteca compartida.

pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;

use axum::Router;

use crate::config::AppConfig;
use crate::state::AppState;

/// Construye el router HTTP del simulador Binance.
pub fn build_app(config: AppConfig) -> Router {
    routes::create_router(AppState::new(config))
}
