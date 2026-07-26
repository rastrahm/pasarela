//! Cross-service — Gateway + Oracle real en procesos locales (Fase 6.2).
//!
//! ```text
//! POST /api/v1/checkout → HttpOracleClient → Oracle real (PostgreSQL) → stub settlement
//! ```
//!
//! Requiere PostgreSQL accesible en `ORACLE_DATABASE_URL` (default `oracle_test`).
//! Ejecutar con un solo hilo: `cargo test -p api-gateway --test cross_service_integration -- --test-threads=1`

mod common;

#[allow(unused_imports)]
use common::{
    build_cross_service_app, checkout_payload, cross_service_app_config, get_health,
    get_transaction, post_checkout, test_merchant_id, test_merchant_registry,
};

use std::sync::Arc;

use axum::http::StatusCode;
use domain::{FundingType, LiquidityError};
use oracle_authorization::test_support::{
    count_active_holds, count_holds, postgres_available, spawn_oracle_server,
};
use serial_test::serial;
use settlement_adapters::{MockSettlementAdapter, SettlementAdapter, SettlementEngine};

use api_gateway::services::RailContext;

async fn require_postgres() -> bool {
    if postgres_available().await {
        return true;
    }
    eprintln!(
        "SKIP: PostgreSQL no disponible (ORACLE_DATABASE_URL={})",
        oracle_authorization::test_support::default_test_database_url()
    );
    false
}

#[serial]
#[tokio::test]
async fn cross_service_health_endpoints() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, _pool) = spawn_oracle_server().await;
    let app = build_cross_service_app(oracle_url);

    let (status, json) = get_health(&app).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "api-gateway");
}

#[serial]
#[tokio::test]
async fn cross_service_full_checkout_settled_and_queryable() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, pool) = spawn_oracle_server().await;
    let app = build_cross_service_app(oracle_url);

    let holds_before = count_holds(&pool).await;

    let (status, checkout) = post_checkout(
        &app,
        &checkout_payload("traditional_bank"),
        "cross-service-full-001",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(checkout["status"], "settled");
    assert_eq!(checkout["rail_used"], "traditional_bank");
    assert!(
        checkout["settlement_proof"]
            .as_str()
            .expect("proof")
            .starts_with("ACH-")
    );

    let active_holds = count_active_holds(&pool).await;
    assert!(
        active_holds >= holds_before,
        "hold activo tras settlement exitoso (consume pendiente v1.1)"
    );

    let transaction_id = checkout["transaction_id"]
        .as_str()
        .expect("transaction_id");

    let (get_status, tx) = get_transaction(&app, transaction_id).await;

    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(tx["transaction_id"], transaction_id);
    assert_eq!(tx["status"], "settled");
    assert_eq!(tx["settlement_proof"], checkout["settlement_proof"]);
}

#[serial]
#[tokio::test]
async fn cross_service_three_rails_settle_with_distinct_proofs() {
    if !require_postgres().await {
        return;
    }

    let cases = [
        ("traditional_bank", "ACH-"),
        ("binance_cex", "CEX-MEM-"),
        ("solana_wallet", "SOL-MEM-"),
    ];

    for (funding_type, proof_prefix) in cases {
        let (oracle_url, _pool) = spawn_oracle_server().await;
        let app = build_cross_service_app(oracle_url);

        let (status, json) = post_checkout(
            &app,
            &checkout_payload(funding_type),
            &format!("cross-service-rail-{funding_type}"),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "rail {funding_type}");
        assert_eq!(json["rail_used"], funding_type);
        assert!(
            json["settlement_proof"]
                .as_str()
                .expect("proof")
                .starts_with(proof_prefix),
            "rail {funding_type}"
        );
    }
}

#[serial]
#[tokio::test]
async fn cross_service_insufficient_funds_from_real_oracle() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, pool) = spawn_oracle_server().await;
    let app = build_cross_service_app(oracle_url);

    let holds_before = count_holds(&pool).await;

    let body = r#"{"amount":50000.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111111","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;

    let (status, json) = post_checkout(&app, body, "cross-service-insufficient-001").await;

    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(json["error_code"], "INSUFFICIENT_FUNDS");
    assert_eq!(count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn cross_service_invalid_card_from_real_oracle() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, pool) = spawn_oracle_server().await;
    let app = build_cross_service_app(oracle_url);

    let holds_before = count_holds(&pool).await;

    let body = r#"{"amount":100.0,"currency":"USD","funding_type":"traditional_bank","card":{"pan":"4111111111111112","expiry_month":"12","expiry_year":"2030","cvv":"123","cardholder":"Test User"}}"#;

    let (status, json) = post_checkout(&app, body, "cross-service-invalid-card-001").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["error_code"], "INVALID_CARD");
    assert_eq!(count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn cross_service_hold_released_when_settlement_fails() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, pool) = spawn_oracle_server().await;

    let engine = SettlementEngine::new([Arc::new(
        MockSettlementAdapter::new(FundingType::TraditionalBank, "unused")
            .with_error(LiquidityError::SettlementFailed),
    )
        as Arc<dyn SettlementAdapter>]);

    let merchant_id = test_merchant_id();
    let config = cross_service_app_config(oracle_url.clone(), merchant_id);
    let oracle_client = Arc::new(
        oracle_client::HttpOracleClient::new(
            config.oracle_base_url.clone(),
            config.oracle_api_key.clone(),
            config.oracle_timeout_secs,
        )
        .expect("client"),
    );
    let app = api_gateway::build_app(api_gateway::state::AppState::from_parts(
        config,
        oracle_client,
        engine,
        rail_switcher::RailSwitcher,
        RailContext::default(),
        test_merchant_registry(merchant_id),
    ));

    let (status, json) = post_checkout(
        &app,
        &checkout_payload("traditional_bank"),
        "cross-service-hold-release-001",
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json["error_code"], "INTERNAL_ERROR");

    let row: (String,) =
        sqlx::query_as("SELECT status::text FROM hold ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("hold row");
    assert_eq!(row.0, "released");
}

#[serial]
#[tokio::test]
async fn cross_service_idempotent_replay_single_oracle_authorize() {
    if !require_postgres().await {
        return;
    }
    let (oracle_url, pool) = spawn_oracle_server().await;
    let app = build_cross_service_app(oracle_url);

    let auth_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM authorization_request")
            .fetch_one(&pool)
            .await
            .expect("count auth");

    let body = checkout_payload("binance_cex");

    let (first_status, first) =
        post_checkout(&app, &body, "cross-service-idempotent-001").await;
    let (second_status, second) =
        post_checkout(&app, &body, "cross-service-idempotent-001").await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(first["transaction_id"], second["transaction_id"]);

    let auth_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM authorization_request")
            .fetch_one(&pool)
            .await
            .expect("count auth");

    assert_eq!(
        auth_after - auth_before,
        1,
        "Oracle authorize solo una vez por Idempotency-Key"
    );
}
