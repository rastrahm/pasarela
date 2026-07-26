//! Contrato JSON — fixtures canónicos deserializan al DTO del Gateway (DT-P1-01 / 6.8).

mod common;

use api_gateway::routes::CheckoutRequest;

use common::{
    checkout_amount_200, checkout_bank, checkout_binance, checkout_invalid_amount,
    checkout_no_rail, checkout_solana,
};

fn assert_parses(label: &str, json: &str) {
    serde_json::from_str::<CheckoutRequest>(json)
        .unwrap_or_else(|err| panic!("fixture {label} no deserializa a CheckoutRequest: {err}"));
}

#[test]
fn canonical_checkout_fixtures_match_gateway_dto() {
    assert_parses("checkout-bank", &checkout_bank());
    assert_parses("checkout-binance", &checkout_binance());
    assert_parses("checkout-solana", &checkout_solana());
    assert_parses("checkout-no-rail", &checkout_no_rail());
    assert_parses("checkout-invalid-amount", &checkout_invalid_amount());
    assert_parses("checkout-amount-200", &checkout_amount_200());
}

#[test]
fn canonical_fixtures_use_four_digit_expiry_year() {
    for (name, json) in [
        ("bank", checkout_bank()),
        ("binance", checkout_binance()),
        ("solana", checkout_solana()),
    ] {
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(
            value["card"]["expiry_year"], "2030",
            "fixture {name} debe usar expiry_year canónico 2030"
        );
    }
}
