//! Algoritmo de selección de riel según §4.3 Casos de Uso.

use domain::{FundingType, RailError};

use crate::config::{RailAvailability, RailConfig, RailPreference, RailSelectionInput};

/// Motor de decisión de riel con fallback automático por prioridad (D3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RailSwitcher;

impl RailSwitcher {
    /// Selecciona el riel activo para una solicitud de pago.
    ///
    /// # Inputs
    /// - `input`: monto, moneda, preferencias, configuración y disponibilidad por riel.
    ///
    /// # Returns
    /// `FundingType` seleccionado o [`RailError`] si ningún riel es viable.
    pub fn select(&self, input: &RailSelectionInput<'_>) -> Result<FundingType, RailError> {
        let candidates = ordered_candidates(input.preference);

        if candidates.is_empty() {
            return Err(RailError::NoRailAvailable);
        }

        let mut tried_preferred = false;

        for (index, rail) in candidates.iter().enumerate() {
            let is_primary = index == 0;
            if is_primary {
                tried_preferred = true;
            } else if tried_preferred && !input.preference.fallback_enabled {
                return Err(RailError::PreferredUnavailableNoFallback { rail: candidates[0] });
            }

            match self.evaluate_rail(*rail, input) {
                Ok(selected) => return Ok(selected),
                Err(err) if is_primary && !input.preference.fallback_enabled => return Err(err),
                Err(_) => continue,
            }
        }

        Err(RailError::NoRailAvailable)
    }

    fn evaluate_rail(
        &self,
        rail: FundingType,
        input: &RailSelectionInput<'_>,
    ) -> Result<FundingType, RailError> {
        let config = find_config(input.configs, rail).ok_or(RailError::RailDisabled { rail })?;

        if !config.enabled {
            return Err(RailError::RailDisabled { rail });
        }

        let availability = find_availability(input.availability, rail)
            .ok_or(RailError::Unavailable { rail })?;

        if !availability.operational {
            return Err(RailError::Unavailable { rail });
        }

        if !availability.funds_sufficient {
            return Err(RailError::InsufficientFunds { rail });
        }

        if !input.amount.is_positive() {
            return Err(RailError::Unavailable { rail });
        }

        Ok(rail)
    }

    /// Selecciona el riel viable de menor costo efectivo entre los habilitados.
    ///
    /// Útil cuando varios rieles pasan disponibilidad y se desempata por costo.
    pub fn select_lowest_cost(
        &self,
        input: &RailSelectionInput<'_>,
    ) -> Result<FundingType, RailError> {
        let mut viable: Vec<(&RailConfig, FundingType)> = Vec::new();

        for config in input.configs {
            if !config.enabled {
                continue;
            }

            let Some(availability) = find_availability(input.availability, config.funding_type) else {
                continue;
            };

            if availability.is_viable() && input.amount.is_positive() {
                viable.push((config, config.funding_type));
            }
        }

        if viable.is_empty() {
            return self.select(input);
        }

        viable.sort_by(|(left, _), (right, _)| {
            left.effective_cost_percent()
                .cmp(&right.effective_cost_percent())
                .then_with(|| left.priority.cmp(&right.priority))
        });

        Ok(viable[0].1)
    }
}

fn ordered_candidates(preference: &RailPreference) -> Vec<FundingType> {
    let mut ordered = Vec::new();

    if let Some(rail) = preference.preferred_rail {
        push_unique(&mut ordered, rail);
    } else if let Some(rail) = preference.merchant_default {
        push_unique(&mut ordered, rail);
    }

    for rail in default_priority_order() {
        push_unique(&mut ordered, rail);
    }

    ordered
}

fn default_priority_order() -> [FundingType; 3] {
    [
        FundingType::TraditionalBank,
        FundingType::BinanceCex,
        FundingType::SolanaWallet,
    ]
}

fn push_unique(list: &mut Vec<FundingType>, rail: FundingType) {
    if !list.contains(&rail) {
        list.push(rail);
    }
}

fn find_config(configs: &[RailConfig], rail: FundingType) -> Option<&RailConfig> {
    configs.iter().find(|config| config.funding_type == rail)
}

fn find_availability(
    availability: &[RailAvailability],
    rail: FundingType,
) -> Option<&RailAvailability> {
    availability.iter().find(|entry| entry.rail == rail)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::{Amount, Currency};
    use rust_decimal::Decimal;

    use super::*;
    use crate::config::{RailAvailability, RailConfig, RailPreference, RailSelectionInput};

    fn sample_input<'a>(
        preference: &'a RailPreference,
        configs: &'a [RailConfig],
        availability: &'a [RailAvailability],
        currency: &'a Currency,
    ) -> RailSelectionInput<'a> {
        RailSelectionInput {
            amount: Amount::from_units(100),
            currency,
            preference,
            configs,
            availability,
        }
    }

    fn all_enabled_configs() -> Vec<RailConfig> {
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

    fn all_available() -> Vec<RailAvailability> {
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

    fn test_currency() -> Currency {
        Currency::from_str("USD").expect("currency")
    }

    #[test]
    fn selects_explicit_preference_when_viable() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: Some(FundingType::BinanceCex),
            merchant_default: None,
            fallback_enabled: true,
        };
        let configs = all_enabled_configs();
        let availability = all_available();
        let input = sample_input(&preference, &configs, &availability, &currency);

        let selected = RailSwitcher.select(&input).expect("selected");
        assert_eq!(selected, FundingType::BinanceCex);
    }

    #[test]
    fn uses_merchant_default_without_explicit_preference() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: None,
            merchant_default: Some(FundingType::SolanaWallet),
            fallback_enabled: true,
        };
        let configs = all_enabled_configs();
        let availability = all_available();
        let input = sample_input(&preference, &configs, &availability, &currency);

        let selected = RailSwitcher.select(&input).expect("selected");
        assert_eq!(selected, FundingType::SolanaWallet);
    }

    #[test]
    fn falls_back_when_preferred_rail_lacks_funds() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: Some(FundingType::TraditionalBank),
            merchant_default: None,
            fallback_enabled: true,
        };
        let configs = all_enabled_configs();
        let availability = vec![
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
        ];
        let input = sample_input(&preference, &configs, &availability, &currency);

        let selected = RailSwitcher.select(&input).expect("selected");
        assert_eq!(selected, FundingType::BinanceCex);
    }

    #[test]
    fn rejects_when_no_rail_is_viable() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: Some(FundingType::TraditionalBank),
            merchant_default: None,
            fallback_enabled: true,
        };
        let configs = all_enabled_configs();
        let availability = vec![
            RailAvailability {
                rail: FundingType::TraditionalBank,
                operational: true,
                funds_sufficient: false,
            },
            RailAvailability {
                rail: FundingType::BinanceCex,
                operational: false,
                funds_sufficient: true,
            },
            RailAvailability {
                rail: FundingType::SolanaWallet,
                operational: true,
                funds_sufficient: false,
            },
        ];
        let input = sample_input(&preference, &configs, &availability, &currency);

        assert_eq!(
            RailSwitcher.select(&input),
            Err(RailError::NoRailAvailable)
        );
    }

    #[test]
    fn rejects_when_fallback_disabled_and_preferred_fails() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: Some(FundingType::TraditionalBank),
            merchant_default: None,
            fallback_enabled: false,
        };
        let configs = all_enabled_configs();
        let availability = vec![
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
        ];
        let input = sample_input(&preference, &configs, &availability, &currency);

        assert_eq!(
            RailSwitcher.select(&input),
            Err(RailError::InsufficientFunds {
                rail: FundingType::TraditionalBank
            })
        );
    }

    #[test]
    fn skips_disabled_rails_in_config() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: Some(FundingType::TraditionalBank),
            merchant_default: None,
            fallback_enabled: true,
        };
        let configs = vec![
            RailConfig {
                funding_type: FundingType::TraditionalBank,
                enabled: false,
                priority: 1,
                fee_percent: Decimal::ZERO,
                spread_buffer: Decimal::ZERO,
            },
            RailConfig {
                funding_type: FundingType::BinanceCex,
                enabled: true,
                priority: 2,
                fee_percent: Decimal::ZERO,
                spread_buffer: Decimal::ZERO,
            },
        ];
        let availability = vec![
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
        ];
        let input = sample_input(&preference, &configs, &availability, &currency);

        let selected = RailSwitcher.select(&input).expect("selected");
        assert_eq!(selected, FundingType::BinanceCex);
    }

    #[test]
    fn select_lowest_cost_picks_cheapest_viable_rail() {
        let currency = test_currency();
        let preference = RailPreference {
            preferred_rail: None,
            merchant_default: None,
            fallback_enabled: true,
        };
        let configs = all_enabled_configs();
        let availability = all_available();
        let input = sample_input(&preference, &configs, &availability, &currency);

        let selected = RailSwitcher
            .select_lowest_cost(&input)
            .expect("selected");
        assert_eq!(selected, FundingType::TraditionalBank);
    }
}
