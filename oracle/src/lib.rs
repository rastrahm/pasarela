//! Oracle de autorización off-chain — biblioteca compartida con tests de integración.

pub mod auth;
pub mod config;
pub mod error;
pub mod funds;
pub mod routes;
pub mod validation;

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;

/// Construye el router HTTP con rutas públicas e internas protegidas.
///
/// # Inputs
/// - `config`: configuración cargada desde variables de entorno.
///
/// # Returns
/// Router de Axum listo para servir con `axum::serve`.
pub fn build_app(config: AppConfig) -> Router {
    let shared = Arc::new(config);
    routes::create_router(shared)
}
