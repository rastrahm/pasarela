//! Reglas de scoring antifraude simuladas.

use rust_decimal::Decimal;

use crate::config::AppConfig;

/// Solicitud de scoring (contrato compartido con Oracle).
#[derive(Debug, Clone)]
pub struct ScoreInput {
    pub amount: Decimal,
    pub token_hash: String,
}

/// Resultado del motor de reglas.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreResult {
    pub approved: bool,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Evalúa reglas básicas: monto máximo, lista de bloqueo y score sintético.
///
/// # Inputs
/// - `config`: umbrales configurables por entorno.
/// - `input`: monto y token hash (sin PAN).
///
/// # Returns
/// Resultado de scoring con razones de decline si aplica.
pub fn evaluate_score(config: &AppConfig, input: &ScoreInput) -> ScoreResult {
    let mut reasons = Vec::new();
    let mut score: f64 = 0.1;

    if input.amount > config.max_amount {
        reasons.push("amount_exceeds_max".to_string());
        score = score.max(0.95);
    }

    if config.blocked_token_hashes.iter().any(|blocked| blocked == &input.token_hash) {
        reasons.push("token_hash_blocked".to_string());
        score = score.max(0.99);
    }

    let amount_factor = (input.amount / config.max_amount)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0)
        * 0.3;
    score = (score + amount_factor).min(1.0);

    let approved = reasons.is_empty() && score < config.decline_score_threshold;

    if !approved && reasons.is_empty() {
        reasons.push("score_above_threshold".to_string());
    }

    ScoreResult {
        approved,
        score,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn test_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8082,
            api_key: "key".to_string(),
            max_amount: Decimal::from_str("1000").expect("decimal"),
            blocked_token_hashes: vec!["tok_blocked".to_string()],
            decline_score_threshold: 0.85,
        }
    }

    #[test]
    fn approves_small_amount() {
        let result = evaluate_score(
            &test_config(),
            &ScoreInput {
                amount: Decimal::from_str("100").expect("decimal"),
                token_hash: "tok_ok".to_string(),
            },
        );
        assert!(result.approved);
    }

    #[test]
    fn declines_amount_above_max() {
        let result = evaluate_score(
            &test_config(),
            &ScoreInput {
                amount: Decimal::from_str("5000").expect("decimal"),
                token_hash: "tok_ok".to_string(),
            },
        );
        assert!(!result.approved);
        assert!(result.reasons.contains(&"amount_exceeds_max".to_string()));
    }

    #[test]
    fn declines_blocked_token_hash() {
        let result = evaluate_score(
            &test_config(),
            &ScoreInput {
                amount: Decimal::from_str("10").expect("decimal"),
                token_hash: "tok_blocked".to_string(),
            },
        );
        assert!(!result.approved);
        assert!(result.reasons.contains(&"token_hash_blocked".to_string()));
    }
}
