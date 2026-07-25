//! Evaluación de fondos y gestión de holds por riel.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// Tipo de riel de fondeo / liquidación.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FundingType {
    TraditionalBank,
    BinanceCex,
    SolanaWallet,
}

/// Estado de evaluación de fondos.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FundStatus {
    pub sufficient: bool,
    pub available_amount: f64,
    pub currency: String,
}

/// Hold temporal sobre fondos del riel activo.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HoldRecord {
    pub hold_id: Uuid,
    pub funding_type: FundingType,
    pub amount: f64,
    pub currency: String,
}

/// Evalúa disponibilidad de fondos según el riel activo (simulado en Fase 2).
///
/// # Inputs
/// - `amount`: monto solicitado.
/// - `currency`: moneda del pago.
/// - `funding_type`: riel a consultar.
///
/// # Returns
/// `FundStatus` con saldo simulado o error si el riel no responde.
pub fn evaluate_funds(
    amount: f64,
    currency: &str,
    funding_type: FundingType,
) -> Result<FundStatus, AppError> {
    let available = simulated_balance(funding_type);
    let spread_adjusted = apply_spread_buffer(available, funding_type);

    Ok(FundStatus {
        sufficient: spread_adjusted >= amount,
        available_amount: spread_adjusted,
        currency: currency.to_string(),
    })
}

/// Crea un hold preventivo sobre el monto autorizado.
///
/// # Inputs
/// - `amount`, `currency`, `funding_type`: datos del hold.
///
/// # Returns
/// `HoldRecord` con identificador único o error si fondos insuficientes.
pub fn create_hold(
    amount: f64,
    currency: &str,
    funding_type: FundingType,
) -> Result<HoldRecord, AppError> {
    let status = evaluate_funds(amount, currency, funding_type)?;
    if !status.sufficient {
        return Err(AppError::InsufficientFunds);
    }

    Ok(HoldRecord {
        hold_id: Uuid::new_v4(),
        funding_type,
        amount,
        currency: currency.to_string(),
    })
}

fn simulated_balance(funding_type: FundingType) -> f64 {
    match funding_type {
        FundingType::TraditionalBank => 10_000.0,
        FundingType::BinanceCex => 5_000.0,
        FundingType::SolanaWallet => 2_500.0,
    }
}

fn apply_spread_buffer(balance: f64, funding_type: FundingType) -> f64 {
    match funding_type {
        FundingType::BinanceCex => balance * 0.98,
        _ => balance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sufficient_funds_for_small_amount() {
        let status = evaluate_funds(100.0, "USD", FundingType::TraditionalBank)
            .expect("evaluate");
        assert!(status.sufficient);
    }

    #[test]
    fn insufficient_funds_for_large_amount() {
        let status = evaluate_funds(999_999.0, "USD", FundingType::SolanaWallet)
            .expect("evaluate");
        assert!(!status.sufficient);
    }

    #[test]
    fn create_hold_rejects_insufficient() {
        let result = create_hold(999_999.0, "USD", FundingType::BinanceCex);
        assert!(matches!(result, Err(AppError::InsufficientFunds)));
    }
}
