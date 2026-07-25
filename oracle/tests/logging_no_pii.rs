//! Tests de integración — logs de autorización sin PII.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_authorization::logging::contains_forbidden_pii;
use tower::ServiceExt;

const TEST_PAN: &str = "4111111111111111";

#[test]
fn card_payload_debug_is_redacted() {
    use oracle_authorization::validation::CardPayload;

    let card = CardPayload {
        pan: TEST_PAN.to_string(),
        expiry_month: "12".to_string(),
        expiry_year: "30".to_string(),
        cvv: "123".to_string(),
        cardholder: "Demo User".to_string(),
    };

    let rendered = format!("{card:?}");
    assert!(!rendered.contains(TEST_PAN));
    assert!(!rendered.contains("Demo User"));
    assert!(rendered.contains("[REDACTED]"));
}

#[test]
fn audit_log_detail_has_no_pan_after_authorize() {
    let handle = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    handle.block_on(async {
        let app = common::setup_app().await;

        let body = common::sample_authorize_body(
            "ee0e8400-e29b-41d4-a716-446655440001",
            75.0,
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/v1/authorize")
                    .header("content-type", "application/json")
                    .header("x-api-key", "test-secret-key")
                    .body(Body::from(body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    });

    let pool = handle.block_on(async {
        let config = common::test_config();
        oracle_authorization::init_database(&config.database_url)
            .await
            .expect("postgres")
    });

    let rows: Vec<(String,)> = handle
        .block_on(async {
            sqlx::query_as("SELECT detail FROM oracle_audit_log ORDER BY occurred_at DESC LIMIT 5")
                .fetch_all(&pool)
                .await
        })
        .expect("audit rows");

    for (detail,) in rows {
        assert!(
            !detail.contains(TEST_PAN),
            "audit log contiene PAN: {detail}"
        );
        assert!(
            !contains_forbidden_pii(&detail),
            "audit log contiene PII: {detail}"
        );
    }
}
