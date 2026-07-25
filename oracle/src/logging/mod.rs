//! Logging estructurado sin PII (Plan 2.8).

mod capture;
mod events;
mod http;
mod redact;

pub use capture::run_with_log_capture;
pub use events::{
    authorization_approved, authorization_rejected, authorization_started,
    dependency_unavailable, gateway_access_denied, hold_released, invalid_card_attempt,
    rate_limit_exceeded,
};
pub use http::http_trace_layer;
pub use redact::{contains_forbidden_pii, RedactedCard, FORBIDDEN_LOG_FIELDS};

use tracing_subscriber::EnvFilter;

/// Inicializa el subscriber global de tracing para el proceso.
pub fn init_subscriber() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
