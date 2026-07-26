//! Utilidades de test compartidas — Oracle real en procesos locales (Fase 6.2).
//!
//! Expuesto con la feature `test-utils` para suites cross-service del Gateway.

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::antifraud_client::{AntifraudClient, MockAntifraudClient};
use crate::config::{AppConfig, CallerRule};
use crate::persistence::AppState;
use crate::rail_adapters::{MockRailBalanceProvider, RailBalanceProvider};
use crate::{build_app, init_database};

/// API key usada en configuración de test del Oracle.
pub const TEST_ORACLE_API_KEY: &str = "test-secret-key";

/// URL PostgreSQL por defecto para tests de integración.
pub fn default_test_database_url() -> String {
    std::env::var("ORACLE_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/oracle_test".to_string()
    })
}

/// Comprueba si PostgreSQL de test está accesible.
pub async fn postgres_available() -> bool {
    init_database(&default_test_database_url())
        .await
        .is_ok()
}

/// Configuración de prueba con saldos reducidos para casos de fondos insuficientes.
pub fn test_config() -> AppConfig {
    test_config_with_rate_limit(100)
}

/// Configuración de prueba con límite de tasa personalizado.
pub fn test_config_with_rate_limit(rate_limit_per_minute: u32) -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_url: default_test_database_url(),
        api_key: TEST_ORACLE_API_KEY.to_string(),
        allowed_callers: vec![CallerRule::Exact(
            "127.0.0.1".parse().expect("ip"),
        )],
        rate_limit_per_minute,
        rail_timeout_secs: 5,
        hold_ttl_secs: 300,
        ttl_cleanup_interval_secs: 60,
        traditional_bank_balance: rust_decimal::Decimal::new(10_000, 0),
        binance_cex_balance: rust_decimal::Decimal::new(5_000, 0),
        solana_wallet_balance: rust_decimal::Decimal::new(2_500, 0),
        binance_spread_buffer_pct: rust_decimal::Decimal::new(2, 2),
        binance_cex_base_url: "http://127.0.0.1:8083".to_string(),
        binance_cex_api_key: "test-binance-key".to_string(),
        solana_rpc_url: "http://127.0.0.1:8899".to_string(),
        solana_wallet_pubkey: "DemoWallet1111111111111111111111111111111".to_string(),
        solana_token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        antifraud_base_url: "http://127.0.0.1:8082".to_string(),
        antifraud_api_key: "test-antifraud-key".to_string(),
        antifraud_timeout_secs: 5,
    }
}

/// Inicializa BD, estado y router con mock antifraude que aprueba.
pub async fn setup_app() -> Router {
    setup_app_with_config(test_config()).await
}

/// Inicializa BD y router con configuración personalizada.
pub async fn setup_app_with_config(config: AppConfig) -> Router {
    setup_app_with_antifraud(Arc::new(MockAntifraudClient::approve()), config).await
}

/// Inicializa BD y router con un cliente antifraude inyectado.
pub async fn setup_app_with_antifraud(
    antifraud_client: Arc<dyn AntifraudClient>,
    config: AppConfig,
) -> Router {
    setup_app_with_clients(
        antifraud_client,
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await
}

/// Inicializa BD y router con clientes antifraude y riel inyectados.
pub async fn setup_app_with_clients(
    antifraud_client: Arc<dyn AntifraudClient>,
    rail_provider: Arc<dyn RailBalanceProvider>,
    config: AppConfig,
) -> Router {
    setup_app_with_clients_and_pool(antifraud_client, rail_provider, config)
        .await
        .0
}

/// Igual que `setup_app_with_clients` pero retorna el pool para asserts en BD.
pub async fn setup_app_with_clients_and_pool(
    antifraud_client: Arc<dyn AntifraudClient>,
    rail_provider: Arc<dyn RailBalanceProvider>,
    config: AppConfig,
) -> (Router, PgPool) {
    dotenvy::dotenv().ok();
    let config = Arc::new(config);
    let pool = init_database(&config.database_url)
        .await
        .expect("conectar a PostgreSQL de test — ver ORACLE_DATABASE_URL");

    truncate_test_tables(&pool).await;

    let state = AppState::with_clients(config, pool.clone(), antifraud_client, rail_provider);
    (build_app(state), pool)
}

/// Levanta el Oracle en un puerto efímero para tests HTTP cross-service (Gateway ↔ Oracle).
pub async fn spawn_oracle_server() -> (String, PgPool) {
    spawn_oracle_server_with_config(test_config()).await
}

/// Levanta el Oracle con configuración personalizada.
pub async fn spawn_oracle_server_with_config(config: AppConfig) -> (String, PgPool) {
    let (app, pool) = setup_app_with_clients_and_pool(
        Arc::new(MockAntifraudClient::approve()),
        Arc::new(MockRailBalanceProvider::from_config(&config)),
        config,
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind oracle test server");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve oracle test server");
    });
    (format!("http://{addr}"), pool)
}

/// Limpia tablas de test — expuesto para suites que arman AppState manualmente.
pub async fn truncate_test_tables_public(pool: &PgPool) {
    truncate_test_tables(pool).await;
}

async fn truncate_test_tables(pool: &PgPool) {
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    static TRUNCATE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = TRUNCATE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .await;

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

/// Configuración con allowlist CIDR para tests de red interna Docker.
pub fn test_config_with_cidr_allowlist() -> AppConfig {
    let mut config = test_config();
    config.allowed_callers = vec![
        CallerRule::Exact("127.0.0.1".parse().expect("ip")),
        CallerRule::Prefix {
            base: "172.17.0.0".parse().expect("ip"),
            prefix_len: 16,
        },
    ];
    config
}

/// Pool PostgreSQL de test (misma URL que `test_config`).
pub async fn test_pool() -> PgPool {
    let config = test_config();
    init_database(&config.database_url)
        .await
        .expect("conectar a PostgreSQL de test — ver ORACLE_DATABASE_URL")
}

/// Cuenta holds con estado `active`.
pub async fn count_active_holds(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM hold WHERE status = 'active'")
        .fetch_one(pool)
        .await
        .expect("contar holds activos")
}

/// Cuenta filas en `hold` (cualquier estado).
pub async fn count_holds(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM hold")
        .fetch_one(pool)
        .await
        .expect("contar holds")
}
