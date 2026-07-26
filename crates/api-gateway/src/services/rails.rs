//! Configuración por defecto de rieles para el Rail Switcher.

use domain::FundingType;
use rail_switcher::{RailAvailability, RailConfig, RailPreference, RailSelectionInput, RailSwitcher};
use rust_decimal::Decimal;

use crate::error::GatewayError;
use crate::routes::CheckoutRequest;

/// Selecciona el riel activo según preferencia del checkout y reglas D3.
pub fn select_rail(
    switcher: &RailSwitcher,
    request: &CheckoutRequest,
    amount: domain::Amount,
    currency: &domain::Currency,
) -> Result<FundingType, GatewayError> {
    let preference = RailPreference {
        preferred_rail: request.funding_type,
        merchant_default: None,
        fallback_enabled: true,
    };

    let input = RailSelectionInput {
        amount,
        currency,
        preference: &preference,
        configs: &default_rail_configs(),
        availability: &default_availability(),
    };

    switcher
        .select(&input)
        .map_err(|_| GatewayError::RailUnavailable)
}

/// Configuración operativa por defecto (tres rieles habilitados).
pub fn default_rail_configs() -> Vec<RailConfig> {
    vec![
        RailConfig {
            funding_type: FundingType::TraditionalBank,
            enabled: true,
            priority: 1,
            fee_percent: Decimal::new(1, 1),
            spread_buffer: Decimal::ZERO,
        },
        RailConfig {
            funding_type: FundingType::BinanceCex,
            enabled: true,
            priority: 2,
            fee_percent: Decimal::new(15, 1),
            spread_buffer: Decimal::new(2, 2),
        },
        RailConfig {
            funding_type: FundingType::SolanaWallet,
            enabled: true,
            priority: 3,
            fee_percent: Decimal::new(5, 1),
            spread_buffer: Decimal::ZERO,
        },
    ]
}

fn default_availability() -> Vec<RailAvailability> {
    vec![
        RailAvailability {
            rail: FundingType::TraditionalBank,
            operational: true,
            funds_sufficient: true,
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
    ]
}

/// Identificador on-chain / ER del riel (`PaymentProcessed.settlement_rail_id`).
pub fn settlement_rail_id(rail: FundingType) -> u64 {
    match rail {
        FundingType::TraditionalBank => 1,
        FundingType::BinanceCex => 2,
        FundingType::SolanaWallet => 3,
    }
}

#[cfg(test)]
mod tests {
    use domain::Currency;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn selects_explicit_preference() {
        let switcher = RailSwitcher;
        let request = CheckoutRequest {
            amount: 100.0,
            currency: "USD".to_string(),
            card: crate::routes::CheckoutCardPayload {
                pan: "4111111111111111".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "2030".to_string(),
                cvv: "123".to_string(),
                cardholder: "Test".to_string(),
            },
            funding_type: Some(FundingType::BinanceCex),
        };
        let currency = Currency::from_str("USD").expect("currency");
        let rail = select_rail(
            &switcher,
            &request,
            domain::Amount::from_units(100),
            &currency,
        )
        .expect("rail");
        assert_eq!(rail, FundingType::BinanceCex);
    }
}
