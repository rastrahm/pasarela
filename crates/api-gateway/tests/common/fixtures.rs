//! Carga fixtures JSON canónicos desde `scripts/fixtures/`.

use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/fixtures")
}

/// Lee un fixture por nombre de archivo (ej. `checkout-bank.json`).
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name))
        .unwrap_or_else(|err| panic!("no se pudo leer fixture {name}: {err}"))
}

pub fn checkout_bank() -> String {
    load_fixture("checkout-bank.json")
}

pub fn checkout_binance() -> String {
    load_fixture("checkout-binance.json")
}

pub fn checkout_solana() -> String {
    load_fixture("checkout-solana.json")
}

pub fn checkout_no_rail() -> String {
    load_fixture("checkout-no-rail.json")
}

pub fn checkout_invalid_amount() -> String {
    load_fixture("checkout-invalid-amount.json")
}

pub fn checkout_amount_200() -> String {
    load_fixture("checkout-amount-200.json")
}

/// Payload JSON de checkout con riel explícito (fixture canónico por riel).
pub fn checkout_payload(funding_type: &str) -> String {
    match funding_type {
        "traditional_bank" => checkout_bank(),
        "binance_cex" => checkout_binance(),
        "solana_wallet" => checkout_solana(),
        other => panic!("fixture no definido para riel: {other}"),
    }
}

/// Payload JSON de checkout sin preferencia de riel.
pub fn checkout_payload_no_rail() -> String {
    checkout_no_rail()
}
