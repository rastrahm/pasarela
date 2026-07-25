//! Rutas HTTP del servicio antifraude.

use std::sync::Arc;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::require_api_key;
use crate::config::AppConfig;
use crate::rules::{evaluate_score, ScoreInput};

/// Crea el router con healthcheck y score interno protegido.
pub fn create_router(config: Arc<AppConfig>) -> Router {
    let public = Router::new().route("/health", get(health));

    let internal = Router::new()
        .route("/internal/v1/score", post(score))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            require_api_key,
        ));

    Router::new()
        .merge(public)
        .merge(internal)
        .with_state(config)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "antifraud-service",
    })
}

#[derive(Debug, Deserialize)]
struct ScoreRequestBody {
    gateway_request_id: Uuid,
    amount: Decimal,
    currency: String,
    funding_type: String,
    token_hash: String,
}

#[derive(Debug, Serialize)]
struct ScoreResponseBody {
    approved: bool,
    score: f64,
    reasons: Vec<String>,
}

/// Evalúa scoring antifraude para el Oracle (UC-12).
async fn score(
    State(config): State<Arc<AppConfig>>,
    Json(body): Json<ScoreRequestBody>,
) -> Json<ScoreResponseBody> {
    let result = evaluate_score(
        &config,
        &ScoreInput {
            amount: body.amount,
            token_hash: body.token_hash,
        },
    );

    tracing::info!(
        gateway_request_id = %body.gateway_request_id,
        amount = %body.amount,
        currency = %body.currency,
        funding_type = %body.funding_type,
        approved = result.approved,
        score = result.score,
        "score antifraude evaluado"
    );

    Json(ScoreResponseBody {
        approved: result.approved,
        score: result.score,
        reasons: result.reasons,
    })
}
