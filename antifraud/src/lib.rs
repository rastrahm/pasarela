//! Servicio antifraude simulado — biblioteca compartida.

pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod rules;

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;

/// Construye el router HTTP del servicio antifraude.
pub fn build_app(config: AppConfig) -> Router {
    routes::create_router(Arc::new(config))
}
