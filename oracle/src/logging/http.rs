//! Capa HTTP de tracing sin registrar cuerpos ni headers sensibles.

use axum::http::Request;
use axum::response::Response;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Level;

/// Crea la capa de tracing HTTP segura para el Oracle.
///
/// Registra método, ruta, latencia y código de estado.
/// No registra cuerpos, headers ni query strings (evita fugas de PAN/CVV).
pub fn http_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&Request<axum::body::Body>) -> tracing::Span + Clone,
    impl Fn(&Request<axum::body::Body>, &tracing::Span) + Clone,
    impl Fn(&Response, std::time::Duration, &tracing::Span) + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                path = %request.uri().path(),
            )
        })
        .on_request(|_request: &Request<_>, _span: &tracing::Span| {
            // Sin headers ni body — el PAN viaja en JSON del authorize.
        })
        .on_response(
            |response: &Response, latency: std::time::Duration, _span: &tracing::Span| {
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
