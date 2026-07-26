//! Generación simulada de mensajes ISO 20022 pacs.008 / compensación ACH.

use domain::{Amount, FundingType, HoldId, LiquidityError, TransactionId};
use serde::{Deserialize, Serialize};

use crate::context::SettlementContext;

/// Documento pacs.008 simulado para compensación batch (ACH / SEPA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pacs008Document {
    /// Tipo de mensaje ISO 20022.
    pub message_type: String,
    /// Identificador único del mensaje de compensación.
    pub message_id: String,
    /// Marca temporal simulada de creación (RFC 3339).
    pub creation_datetime: String,
    /// Cantidad de transacciones incluidas en el lote.
    pub number_of_transactions: u32,
    /// Suma de control del lote (monto total).
    pub control_sum: String,
    /// Monto instruido con moneda ISO 4217.
    pub instructed_amount: InstructedAmount,
    /// Identificador extremo a extremo (EndToEndId).
    pub end_to_end_id: String,
    /// Referencia al hold off-chain consumido.
    pub hold_reference: String,
    /// Referencia al comercio acreedor.
    pub creditor_reference: String,
    /// Referencia a la transacción del Gateway.
    pub transaction_reference: String,
}

/// Monto instruido en un mensaje pacs.008.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstructedAmount {
    pub currency: String,
    pub value: String,
}

/// Resultado de la compensación bancaria simulada.
#[derive(Debug, Clone, PartialEq)]
pub struct BankCompensation {
    /// Referencia bancaria corta — se usa como `settlement_proof`.
    pub reference_id: String,
    /// Payload JSON del mensaje pacs.008 simulado (auditoría / reconciliación).
    pub iso20022_message: String,
}

/// Genera un archivo de compensación simulado a partir del contexto de liquidación.
///
/// # Inputs
/// - `context`: hold, monto, moneda, comercio y transacción autorizados.
///
/// # Returns
/// [`BankCompensation`] con referencia ACH e ISO 20022 serializado, o error si el monto no es válido.
pub fn generate(context: &SettlementContext) -> Result<BankCompensation, LiquidityError> {
    validate_amount(context.amount)?;

    let reference_id = bank_reference_id(context.hold_id, context.transaction_id);
    let document = build_pacs008(context, &reference_id)?;
    let iso20022_message = serde_json::to_string(&document).map_err(|_| LiquidityError::SettlementFailed)?;

    Ok(BankCompensation {
        reference_id,
        iso20022_message,
    })
}

fn validate_amount(amount: Amount) -> Result<(), LiquidityError> {
    if amount.is_positive() {
        Ok(())
    } else {
        Err(LiquidityError::InsufficientFunds {
            rail: FundingType::TraditionalBank,
        })
    }
}

fn build_pacs008(
    context: &SettlementContext,
    reference_id: &str,
) -> Result<Pacs008Document, LiquidityError> {
    let hold_reference = context.hold_id.0.to_string();
    let transaction_reference = context.transaction_id.to_string();
    let creditor_reference = context.merchant_id.to_string();
    let amount_value = context.amount.value().normalize().to_string();

    Ok(Pacs008Document {
        message_type: "pacs.008.001.08".to_string(),
        message_id: format!("MSG-{reference_id}"),
        creation_datetime: simulated_creation_datetime(context.hold_id),
        number_of_transactions: 1,
        control_sum: amount_value.clone(),
        instructed_amount: InstructedAmount {
            currency: context.currency.as_str().to_string(),
            value: amount_value,
        },
        end_to_end_id: format!("E2E-{transaction_reference}"),
        hold_reference,
        creditor_reference,
        transaction_reference,
    })
}

/// Construye la referencia bancaria ficticia expuesta como prueba de asentamiento.
fn bank_reference_id(hold_id: HoldId, transaction_id: TransactionId) -> String {
    let hold_suffix = short_uuid(hold_id.0);
    let tx_suffix = short_uuid(transaction_id.0);
    format!("ACH-{hold_suffix}-{tx_suffix}")
}

fn short_uuid(id: uuid::Uuid) -> String {
    id.to_string()
        .replace('-', "")
        .chars()
        .take(8)
        .collect::<String>()
        .to_uppercase()
}

/// Timestamp determinista derivado del hold (simulación — no reloj real).
fn simulated_creation_datetime(hold_id: HoldId) -> String {
    let bytes = hold_id.0.as_bytes();
    let day = (bytes[0] % 28) + 1;
    let hour = bytes[1] % 24;
    let minute = bytes[2] % 60;
    format!("2026-07-{day:02}T{hour:02}:{minute:02}:00Z")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};
    use uuid::Uuid;

    use super::*;

    fn sample_context(amount_units: i64) -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::from_u128(0x1111_2222_3333_4444_5555_6666_7777_8888)),
            transaction_id: TransactionId::new(Uuid::from_u128(0xaaaa_bbbb_cccc_dddd_eeee_ffff_0000_1111)),
            merchant_id: MerchantId::new(Uuid::from_u128(0x9999_8888_7777_6666_5555_4444_3333_2222)),
            amount: Amount::from_units(amount_units),
            currency: Currency::from_str("USD").expect("currency"),
            brand_code: 1,
            settlement_rail_id: 1,
        }
    }

    #[test]
    fn generates_ach_reference_with_expected_prefix() {
        let context = sample_context(100);
        let result = generate(&context).expect("compensation");

        assert!(result.reference_id.starts_with("ACH-"));
        assert_eq!(result.reference_id, "ACH-11112222-AAAABBBB");
    }

    #[test]
    fn iso20022_message_contains_amount_currency_and_references() {
        let context = sample_context(250);
        let result = generate(&context).expect("compensation");
        let document: Pacs008Document =
            serde_json::from_str(&result.iso20022_message).expect("valid json");

        assert_eq!(document.message_type, "pacs.008.001.08");
        assert_eq!(document.number_of_transactions, 1);
        assert_eq!(document.control_sum, "250");
        assert_eq!(document.instructed_amount.currency, "USD");
        assert_eq!(document.instructed_amount.value, "250");
        assert_eq!(document.end_to_end_id, format!("E2E-{}", context.transaction_id));
        assert_eq!(document.hold_reference, context.hold_id.0.to_string());
        assert_eq!(document.creditor_reference, context.merchant_id.to_string());
    }

    #[test]
    fn rejects_non_positive_amount() {
        let context = sample_context(0);
        assert_eq!(
            generate(&context),
            Err(LiquidityError::InsufficientFunds {
                rail: FundingType::TraditionalBank
            })
        );

        let negative = SettlementContext {
            amount: Amount::from_units(-10),
            ..sample_context(100)
        };
        assert_eq!(
            generate(&negative),
            Err(LiquidityError::InsufficientFunds {
                rail: FundingType::TraditionalBank
            })
        );
    }

    #[test]
    fn reference_is_deterministic_for_same_context() {
        let context = sample_context(100);
        let first = generate(&context).expect("first").reference_id;
        let second = generate(&context).expect("second").reference_id;
        assert_eq!(first, second);
    }
}
