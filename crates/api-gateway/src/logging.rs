//! Logging estructurado del Gateway (sin PII en trazas).

use axum::http::{Request, Response};
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Inicializa el subscriber global de tracing.
pub fn init_subscriber() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

/// Capa HTTP de tracing — sin registrar cuerpos ni headers sensibles (PAN/CVV).
pub fn http_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&Request<axum::body::Body>) -> tracing::Span + Clone,
    impl Fn(&Request<axum::body::Body>, &tracing::Span) + Clone,
    impl Fn(&Response<axum::body::Body>, std::time::Duration, &tracing::Span) + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                path = %request.uri().path(),
            )
        })
        .on_request(|_request: &Request<_>, _span: &tracing::Span| {})
        .on_response(
            |response: &Response<axum::body::Body>, latency: std::time::Duration, _span: &tracing::Span| {
                tracing::info!(
                    event = "http_response",
                    status = response.status().as_u16(),
                    latency_ms = latency.as_millis() as u64,
                    "respuesta HTTP"
                );
            },
        )
        .on_failure(DefaultOnFailure::new().level(Level::WARN))
}
