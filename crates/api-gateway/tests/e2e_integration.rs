//! E2E — Gateway + mock Oracle + mock rieles (paso 4.14 / gate Fase 4).
//!
//! Valida el flujo completo:
//! ```text
//! POST /api/v1/checkout → Rail Switcher → Oracle /authorize → Settlement → respuesta
//! GET  /api/v1/transactions/{id}
//! ```

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::http::StatusCode;
use domain::{FundingType, LiquidityError};
use oracle_client::FundingType as OracleFundingType;
use rail_switcher::RailAvailability;
use settlement_adapters::{MockSettlementAdapter, SettlementAdapter, SettlementEngine};

use api_gateway::services::RailContext;
use common::{
    build_default_test_app, build_test_app, checkout_payload, checkout_payload_no_rail,
    get_health, get_transaction, post_checkout, post_checkout_unauthenticated,
    post_checkout_without_idempotency, spawn_mock_oracle, MockOracleOpts,
};

#[tokio::test]
async fn gate_health_endpoint_ready() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let (status, json) = get_health(&app).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "api-gateway");
}

#[tokio::test]
async fn gate_full_checkout_flow_settled_and_queryable() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let (status, checkout) =
        post_checkout(&app, &checkout_payload("traditional_bank"), "e2e-full-flow-001").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(checkout["status"], "settled");
    assert_eq!(checkout["rail_used"], "traditional_bank");
    assert!(checkout["settlement_proof"]
        .as_str()
        .expect("proof")
        .starts_with("ACH-"));
    assert_eq!(capture.authorize_calls.load(Ordering::SeqCst), 1);
    assert_eq!(capture.release_calls.load(Ordering::SeqCst), 0);

    let transaction_id = checkout["transaction_id"]
        .as_str()
        .expect("transaction_id");

    let (get_status, tx) = get_transaction(&app, transaction_id).await;

    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(tx["transaction_id"], transaction_id);
    assert_eq!(tx["status"], "settled");
    assert_eq!(tx["rail_used"], "traditional_bank");
    assert_eq!(tx["settlement_proof"], checkout["settlement_proof"]);
}

#[tokio::test]
async fn gate_three_rails_settle_with_distinct_proofs() {
    let cases = [
        ("traditional_bank", "ACH-"),
        ("binance_cex", "CEX-MEM-"),
        ("solana_wallet", "SOL-MEM-"),
    ];

    for (funding_type, proof_prefix) in cases {
        let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
        let app = build_default_test_app(oracle_url);

        let (status, json) = post_checkout(
            &app,
            &checkout_payload(funding_type),
            &format!("e2e-rail-{funding_type}"),
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
        assert_eq!(capture.release_calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn gate_rail_fallback_selects_next_viable_rail() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let rail_context = RailContext::default().with_availability(vec![
        RailAvailability {
            rail: FundingType::TraditionalBank,
            operational: true,
            funds_sufficient: false,
        },
        RailAvailability {
            rail: FundingType::BinanceCex,
            operational: true,
            funds_sufficient: true,
        },
        RailAvailability {
            rail: FundingType::SolanaWallet,
            operational: true,
            funds_sufficient: true,
        },
    ]);
    let app = build_test_app(
        oracle_url,
        rail_context,
        SettlementEngine::with_stub_adapters(),
    );

    let (status, json) = post_checkout(
        &app,
        &checkout_payload("traditional_bank"),
        "e2e-fallback-001",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["rail_used"], "binance_cex");
    assert!(
        json["settlement_proof"]
            .as_str()
            .expect("proof")
            .starts_with("CEX-MEM-")
    );
    assert_eq!(
        *capture.last_funding_type.lock().expect("lock"),
        Some(OracleFundingType::BinanceCex)
    );
}

#[tokio::test]
async fn gate_merchant_default_rail_when_checkout_has_no_preference() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let rail_context = RailContext {
        merchant_default: Some(FundingType::SolanaWallet),
        ..RailContext::default()
    };
    let app = build_test_app(
        oracle_url,
        rail_context,
        SettlementEngine::with_stub_adapters(),
    );

    let (status, json) = post_checkout(
        &app,
        &checkout_payload_no_rail(),
        "e2e-merchant-default-001",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["rail_used"], "solana_wallet");
    assert_eq!(
        *capture.last_funding_type.lock().expect("lock"),
        Some(OracleFundingType::SolanaWallet)
    );
}

#[tokio::test]
async fn gate_hold_released_when_settlement_fails() {
    let hold_id = uuid::Uuid::new_v4();
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts {
        hold_id,
        insufficient_funds: false,
    })
    .await;
    let engine = SettlementEngine::new([Arc::new(
        MockSettlementAdapter::new(FundingType::TraditionalBank, "unused")
            .with_error(LiquidityError::SettlementFailed),
    )
        as Arc<dyn SettlementAdapter>]);
    let app = build_test_app(oracle_url, RailContext::default(), engine);

    let (status, json) = post_checkout(
        &app,
        &checkout_payload("traditional_bank"),
        "e2e-hold-release-001",
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json["error_code"], "INTERNAL_ERROR");
    assert_eq!(capture.release_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        capture.released_hold_ids.lock().expect("lock").as_slice(),
        &[hold_id]
    );
}

#[tokio::test]
async fn gate_checkout_requires_bearer_token() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let status = post_checkout_unauthenticated(
        &app,
        &checkout_payload("traditional_bank"),
        "e2e-no-auth",
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gate_checkout_requires_idempotency_key() {
    let (oracle_url, _) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);

    let status =
        post_checkout_without_idempotency(&app, &checkout_payload("traditional_bank")).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn gate_idempotent_replay_returns_same_transaction() {
    let (oracle_url, capture) = spawn_mock_oracle(MockOracleOpts::default()).await;
    let app = build_default_test_app(oracle_url);
    let body = checkout_payload("binance_cex");

    let (first_status, first) =
        post_checkout(&app, &body, "e2e-idempotent-replay").await;
    let (second_status, second) =
        post_checkout(&app, &body, "e2e-idempotent-replay").await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(first["transaction_id"], second["transaction_id"]);
    assert_eq!(first["settlement_proof"], second["settlement_proof"]);
    assert_eq!(
        capture.authorize_calls.load(Ordering::SeqCst),
        1,
        "Oracle authorize solo una vez"
    );
}
