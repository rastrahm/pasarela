//! Tests de contrato — fixtures alineados con `oracle/openapi/v1.yaml`.

use oracle_client::{
    AuthorizeRequest, AuthorizeResponse, ErrorResponse, ReleaseHoldResponse,
};

#[test]
fn fixture_authorize_request_matches_contract() {
    let json = include_str!("fixtures/authorize_request.json");
    let request: AuthorizeRequest = serde_json::from_str(json).expect("authorize request");
    assert_eq!(request.currency, "USD");
}

#[test]
fn fixture_authorize_response_matches_contract() {
    let json = include_str!("fixtures/authorize_response.json");
    let response: AuthorizeResponse = serde_json::from_str(json).expect("authorize response");
    assert_eq!(response.brand, "visa");
}

#[test]
fn fixture_release_hold_response_matches_contract() {
    let json = include_str!("fixtures/release_hold_response.json");
    let response: ReleaseHoldResponse = serde_json::from_str(json).expect("release response");
    assert_eq!(response.status, "released");
}

#[test]
fn fixture_error_response_matches_contract() {
    let json = include_str!("fixtures/error_response.json");
    let response: ErrorResponse = serde_json::from_str(json).expect("error response");
    assert_eq!(response.error_code, "INSUFFICIENT_FUNDS");
}
