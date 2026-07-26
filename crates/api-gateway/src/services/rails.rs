//! Integración del Rail Switcher — configuración operativa y selección de riel.

use domain::FundingType;
use rail_switcher::{RailAvailability, RailConfig, RailPreference, RailSelectionInput, RailSwitcher};
use rust_decimal::Decimal;

use crate::error::GatewayError;
use crate::routes::CheckoutRequest;

/// Contexto operativo de rieles inyectado en [`crate::state::AppState`].
#[derive(Debug, Clone)]
pub struct RailContext {
    pub configs: Vec<RailConfig>,
    pub availability: Vec<RailAvailability>,
    pub merchant_default: Option<FundingType>,
    pub fallback_enabled: bool,
}

impl Default for RailContext {
    fn default() -> Self {
        Self {
            configs: default_rail_configs(),
            availability: default_availability(),
            merchant_default: None,
            fallback_enabled: true,
        }
    }
}

impl RailContext {
    /// Sustituye el snapshot de disponibilidad (tests / overrides operativos).
    pub fn with_availability(mut self, availability: Vec<RailAvailability>) -> Self {
        self.availability = availability;
        self
    }
}

/// Selecciona el riel activo según preferencia del checkout y reglas D3.
pub fn select_rail(
    switcher: &RailSwitcher,
    context: &RailContext,
    request: &CheckoutRequest,
    amount: domain::Amount,
    currency: &domain::Currency,
) -> Result<FundingType, GatewayError> {
    let preference = RailPreference {
        preferred_rail: request.funding_type,
        merchant_default: context.merchant_default,
        fallback_enabled: context.fallback_enabled,
    };

    let input = RailSelectionInput {
        amount,
        currency,
        preference: &preference,
        configs: &context.configs,
        availability: &context.availability,
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

/// Disponibilidad optimista por defecto hasta consulta Oracle en producción.
pub fn default_availability() -> Vec<RailAvailability> {
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
        let context = RailContext::default();
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
            &context,
            &request,
            domain::Amount::from_units(100),
            &currency,
        )
        .expect("rail");
        assert_eq!(rail, FundingType::BinanceCex);
    }

    #[test]
    fn uses_merchant_default_when_no_explicit_preference() {
        let switcher = RailSwitcher;
        let context = RailContext {
            merchant_default: Some(FundingType::SolanaWallet),
            ..RailContext::default()
        };
        let request = CheckoutRequest {
            amount: 50.0,
            currency: "USD".to_string(),
            card: crate::routes::CheckoutCardPayload {
                pan: "4111111111111111".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "2030".to_string(),
                cvv: "123".to_string(),
                cardholder: "Test".to_string(),
            },
            funding_type: None,
        };
        let currency = Currency::from_str("USD").expect("currency");
        let rail = select_rail(
            &switcher,
            &context,
            &request,
            domain::Amount::from_units(50),
            &currency,
        )
        .expect("rail");
        assert_eq!(rail, FundingType::SolanaWallet);
    }

    #[test]
    fn falls_back_when_preferred_rail_lacks_funds() {
        let switcher = RailSwitcher;
        let context = RailContext::default().with_availability(vec![
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
            funding_type: Some(FundingType::TraditionalBank),
        };
        let currency = Currency::from_str("USD").expect("currency");
        let rail = select_rail(
            &switcher,
            &context,
            &request,
            domain::Amount::from_units(100),
            &currency,
        )
        .expect("rail");
        assert_eq!(rail, FundingType::BinanceCex);
    }
}
