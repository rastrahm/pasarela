//! Estado compartido del middleware de autenticación del Gateway.

use std::sync::Arc;

use crate::config::AppConfig;

use super::rate_limit::SlidingWindowRateLimiter;

/// Configuración y limitador compartidos entre requests internas.
#[derive(Clone)]
pub struct GatewayAuthState {
    pub config: Arc<AppConfig>,
    pub rate_limiter: Arc<SlidingWindowRateLimiter>,
}

impl GatewayAuthState {
    /// Crea el estado de auth a partir de la configuración de la app.
    pub fn from_config(config: Arc<AppConfig>) -> Arc<Self> {
        Arc::new(Self {
            rate_limiter: SlidingWindowRateLimiter::new(config.rate_limit_per_minute),
            config,
        })
    }
}
