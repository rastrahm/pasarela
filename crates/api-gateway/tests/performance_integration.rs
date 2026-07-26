//! Tests de rendimiento — latencia checkout con mocks (Fase 6.5).
//!
//! Valida presupuesto de latencia en el camino feliz con Oracle mock + settlement stub.
//! No sustituye prueba de carga con stack real (ver `Doc/Revision-Rendimiento-Fase-6.md`).

mod common;

use std::time::{Duration, Instant};

use axum::http::StatusCode;

use common::{
    build_default_test_app, checkout_payload, post_checkout, spawn_mock_oracle, MockOracleOpts,
};

/// Techo de latencia p99 local con mocks (Oracle HTTP + settlement in-memory).
/// Margen generoso para CI compartida; stack real tiene presupuesto distinto por riel.
const CHECKOUT_P99_BUDGET: Duration = Duration::from_millis(500);

/// Muestras por riel en el smoke de rendimiento.
const SAMPLES_PER_RAIL: usize = 5;

#[tokio::test]
async fn performance_checkout_stub_path_under_budget_per_rail() {
    let rails = ["traditional_bank", "binance_cex", "solana_wallet"];

    for rail in rails {
        let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
        let app = build_default_test_app(oracle_url);

        let mut samples = Vec::with_capacity(SAMPLES_PER_RAIL);

        for sample in 0..SAMPLES_PER_RAIL {
            let started = Instant::now();
            let (status, _) = post_checkout(
                &app,
                &checkout_payload(rail),
                &format!("perf-{rail}-{sample}"),
            )
            .await;
            let elapsed = started.elapsed();

            assert_eq!(status, StatusCode::OK, "rail {rail} sample {sample}");
            samples.push(elapsed);
        }

        samples.sort();
        let p99_index = samples.len().saturating_sub(1);
        let p99 = samples[p99_index];

        assert!(
            p99 <= CHECKOUT_P99_BUDGET,
            "rail {rail}: p99 {p99:?} excede presupuesto {CHECKOUT_P99_BUDGET:?} (muestras: {samples:?})"
        );
    }
}

#[tokio::test]
async fn performance_health_endpoint_under_50ms() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let started = Instant::now();
    let (status, _) = common::get_health(&app).await;
    let elapsed = started.elapsed();

    assert_eq!(status, StatusCode::OK);
    assert!(
        elapsed <= Duration::from_millis(50),
        "GET /health tardó {elapsed:?}"
    );
}
