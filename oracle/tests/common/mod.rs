//! Utilidades compartidas para tests de integración con PostgreSQL.

use std::sync::Arc;

use axum::Router;
use oracle_authorization::antifraud_client::{AntifraudClient, MockAntifraudClient};
use oracle_authorization::config::{AppConfig, CallerRule};
use oracle_authorization::persistence::AppState;
use oracle_authorization::{build_app, init_database};

/// Configuración de prueba con saldos reducidos para casos de fondos insuficientes.
pub fn test_config() -> AppConfig {
    let database_url = std::env::var("ORACLE_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgre@localhost:5432/oracle_test".to_string()
    });

    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_url,
        api_key: "test-secret-key".to_string(),
        allowed_callers: vec![CallerRule::Exact(
            "127.0.0.1".parse().expect("ip"),
        )],
        rate_limit_per_minute: 100,
        rail_timeout_secs: 5,
        hold_ttl_secs: 300,
        ttl_cleanup_interval_secs: 60,
        traditional_bank_balance: rust_decimal::Decimal::new(10_000, 0),
        binance_cex_balance: rust_decimal::Decimal::new(5_000, 0),
        solana_wallet_balance: rust_decimal::Decimal::new(2_500, 0),
        binance_spread_buffer_pct: rust_decimal::Decimal::new(2, 2),
        antifraud_base_url: "http://127.0.0.1:8082".to_string(),
        antifraud_api_key: "test-antifraud-key".to_string(),
        antifraud_timeout_secs: 5,
    }
}

/// Inicializa BD, estado y router con mock antifraude que aprueba.
pub async fn setup_app() -> Router {
    setup_app_with_antifraud(Arc::new(MockAntifraudClient::approve())).await
}

/// Inicializa BD y router con un cliente antifraude inyectado.
pub async fn setup_app_with_antifraud(antifraud_client: Arc<dyn AntifraudClient>) -> Router {
    dotenvy::dotenv().ok();
    let config = Arc::new(test_config());
    let pool = init_database(&config.database_url)
        .await
        .expect("conectar a PostgreSQL de test — ver ORACLE_DATABASE_URL");

    truncate_test_tables(&pool).await;

    let state = AppState::with_antifraud_client(config, pool, antifraud_client);
    build_app(state)
}

async fn truncate_test_tables(pool: &sqlx::PgPool) {
    sqlx::query(
        r#"
        TRUNCATE TABLE oracle_audit_log, hold, authorization_request
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(pool)
    .await
    .expect("limpiar tablas de test");
}

pub fn sample_authorize_body(gateway_request_id: &str, amount: f64) -> serde_json::Value {
    serde_json::json!({
        "gateway_request_id": gateway_request_id,
        "card": {
            "pan": "4111111111111111",
            "expiry_month": "12",
            "expiry_year": "30",
            "cvv": "123",
            "cardholder": "Demo"
        },
        "amount": amount,
        "currency": "USD",
        "funding_type": "traditional_bank"
    })
}
