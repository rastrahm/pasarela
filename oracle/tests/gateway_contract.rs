//! Tests de contrato Gateway ↔ Oracle — `oracle-client` contra el servicio real (Plan 2.11).
//!
//! Simula el API Gateway invocando al Oracle vía HTTP con DTOs compartidos.
//! Requiere PostgreSQL (`ORACLE_DATABASE_URL`) y `--test-threads=1`.

mod common;

use oracle_client::{
    AuthorizeRequest, CardPayload, FundingType, HttpOracleClient, OracleClient,
    OracleClientError, RequestOptions,
};
use serial_test::serial;
use uuid::Uuid;

const TEST_API_KEY: &str = "test-secret-key";
const GATEWAY_IP: &str = "127.0.0.1";

fn gateway_client(base_url: &str, api_key: &str) -> HttpOracleClient {
    HttpOracleClient::new(base_url.to_string(), api_key.to_string(), 5).expect("cliente gateway")
}

fn gateway_options() -> RequestOptions {
    RequestOptions {
        caller_ip: Some(GATEWAY_IP.to_string()),
    }
}

fn sample_authorize_request(amount: f64) -> AuthorizeRequest {
    AuthorizeRequest {
        gateway_request_id: Uuid::new_v4(),
        card: CardPayload {
            pan: "4111111111111111".to_string(),
            expiry_month: "12".to_string(),
            expiry_year: "30".to_string(),
            cvv: "123".to_string(),
            cardholder: "Demo User".to_string(),
        },
        amount,
        currency: "USD".to_string(),
        funding_type: FundingType::TraditionalBank,
    }
}

#[serial]
#[tokio::test]
async fn gateway_client_health_matches_contract() {
    let (base_url, _pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let health = client.health().await.expect("health");

    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "oracle-authorization");
}

#[serial]
#[tokio::test]
async fn gateway_client_authorize_and_release_full_flow() {
    let (base_url, pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let request = sample_authorize_request(150.0);
    let gateway_id = request.gateway_request_id;

    let auth = client
        .authorize(request, gateway_options())
        .await
        .expect("authorize");

    assert_eq!(auth.gateway_request_id, gateway_id);
    assert_eq!(auth.brand, "visa");
    assert_eq!(auth.brand_code, 1);
    assert_eq!(auth.last_four, "1111");

    let active_before = common::count_active_holds(&pool).await;
    assert!(active_before >= 1);

    let release = client
        .release_hold(auth.hold_id, gateway_options())
        .await
        .expect("release");

    assert_eq!(release.hold_id, auth.hold_id);
    assert_eq!(release.status, "released");

    let row: (String,) = sqlx::query_as("SELECT status::text FROM hold WHERE id = $1")
        .bind(auth.hold_id)
        .fetch_one(&pool)
        .await
        .expect("hold row");
    assert_eq!(row.0, "released");
}

#[serial]
#[tokio::test]
async fn gateway_client_insufficient_funds_maps_contract_error() {
    let (base_url, pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let holds_before = common::count_holds(&pool).await;

    let request = sample_authorize_request(50_000.0);
    let result = client.authorize(request, gateway_options()).await;

    assert_eq!(result, Err(OracleClientError::InsufficientFunds));
    assert_eq!(common::count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn gateway_client_invalid_card_maps_contract_error() {
    let (base_url, pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let holds_before = common::count_holds(&pool).await;

    let mut request = sample_authorize_request(100.0);
    request.card.pan = "4111111111111112".to_string();

    let result = client.authorize(request, gateway_options()).await;

    assert_eq!(result, Err(OracleClientError::InvalidCard));
    assert_eq!(common::count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn gateway_client_unauthorized_with_invalid_api_key() {
    let (base_url, pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, "wrong-key");

    let holds_before = common::count_holds(&pool).await;

    let request = sample_authorize_request(100.0);
    let result = client.authorize(request, gateway_options()).await;

    assert_eq!(result, Err(OracleClientError::Unauthorized));
    assert_eq!(common::count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn gateway_client_forbidden_when_ip_not_in_allowlist() {
    let (base_url, pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let holds_before = common::count_holds(&pool).await;

    let request = sample_authorize_request(100.0);
    let result = client
        .authorize(
            request,
            RequestOptions {
                caller_ip: Some("10.0.0.99".to_string()),
            },
        )
        .await;

    assert_eq!(result, Err(OracleClientError::Forbidden));
    assert_eq!(common::count_holds(&pool).await, holds_before);
}

#[serial]
#[tokio::test]
async fn gateway_client_release_is_idempotent() {
    let (base_url, _pool) = common::spawn_oracle_server().await;
    let client = gateway_client(&base_url, TEST_API_KEY);

    let auth = client
        .authorize(sample_authorize_request(75.0), gateway_options())
        .await
        .expect("authorize");

    let first = client
        .release_hold(auth.hold_id, gateway_options())
        .await
        .expect("first release");
    let second = client
        .release_hold(auth.hold_id, gateway_options())
        .await
        .expect("second release");

    assert_eq!(first.status, "released");
    assert_eq!(second.status, "released");
    assert_eq!(second.hold_id, auth.hold_id);
}
