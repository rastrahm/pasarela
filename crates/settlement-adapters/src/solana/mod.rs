//! Adaptador de liquidación — riel Solana on-chain (`process_payment` + finalized).

mod client;
mod config;
mod instruction;

use std::sync::Arc;

use async_trait::async_trait;
use domain::{FundingType, LiquidityError, SettlementReceipt};

use crate::adapter::SettlementAdapter;
use crate::context::SettlementContext;

pub use client::{
    InMemorySolanaSettlementClient, ProcessPaymentRequest, ProcessPaymentResult,
    RpcSolanaSettlementClient, SolanaClientError, SolanaSettlementClient,
};
pub use config::{SolanaConfigError, SolanaSettlementConfig, amount_to_base_units};
pub use instruction::{
    ProcessPaymentAccounts, build_process_payment_instruction, derive_process_payment_accounts,
    process_payment_discriminator,
};

/// Liquidación vía programa Anchor `payment-settlement`.
#[derive(Clone)]
pub struct SolanaWalletAdapter {
    client: Arc<dyn SolanaSettlementClient>,
    token_decimals: u8,
}

impl SolanaWalletAdapter {
    /// Crea el adaptador con cliente configurable y decimales del mint SPL.
    pub fn new(client: Arc<dyn SolanaSettlementClient>, token_decimals: u8) -> Self {
        Self {
            client,
            token_decimals,
        }
    }

    /// Adaptador con RPC real cargado desde entorno.
    pub fn from_env() -> Result<Self, SolanaConfigError> {
        let config = SolanaSettlementConfig::from_env()?;
        let decimals = config.token_decimals;
        Ok(Self::new(
            Arc::new(RpcSolanaSettlementClient::new(config)),
            decimals,
        ))
    }

    /// Mock en memoria para tests (sin validador ni devnet).
    pub fn in_memory_mock() -> Self {
        Self::new(Arc::new(InMemorySolanaSettlementClient), 6)
    }

    /// Ejecuta `process_payment` y retorna la firma confirmada.
    pub async fn process_payment(
        &self,
        context: &SettlementContext,
    ) -> Result<ProcessPaymentResult, LiquidityError> {
        let amount_base_units = amount_to_base_units(context.amount, self.token_decimals)?;
        let request = ProcessPaymentRequest {
            amount_base_units,
            brand_code: context.brand_code,
            settlement_rail_id: context.settlement_rail_id,
            client_reference: context.hold_id.0.to_string(),
        };

        self.client
            .process_payment(request)
            .await
            .map_err(map_client_error)
    }
}

impl Default for SolanaWalletAdapter {
    fn default() -> Self {
        Self::in_memory_mock()
    }
}

#[async_trait]
impl SettlementAdapter for SolanaWalletAdapter {
    fn rail(&self) -> FundingType {
        FundingType::SolanaWallet
    }

    async fn settle(&self, context: SettlementContext) -> Result<SettlementReceipt, LiquidityError> {
        let result = self.process_payment(&context).await?;

        Ok(SettlementReceipt {
            proof: result.signature,
            rail: FundingType::SolanaWallet,
        })
    }
}

fn map_client_error(error: SolanaClientError) -> LiquidityError {
    match error {
        SolanaClientError::InsufficientFunds => LiquidityError::InsufficientFunds {
            rail: FundingType::SolanaWallet,
        },
        SolanaClientError::InvalidRequest => LiquidityError::SettlementFailed,
        SolanaClientError::FinalizationTimeout => LiquidityError::RailUnavailable {
            rail: FundingType::SolanaWallet,
        },
        SolanaClientError::Unavailable(_) => LiquidityError::RailUnavailable {
            rail: FundingType::SolanaWallet,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use domain::{Amount, Currency, HoldId, MerchantId, TransactionId};
    use uuid::Uuid;

    use super::*;
    use crate::SettlementEngine;

    fn sample_context(amount_units: i64) -> SettlementContext {
        SettlementContext {
            hold_id: HoldId::new(Uuid::new_v4()),
            transaction_id: TransactionId::generate(),
            merchant_id: MerchantId::new(Uuid::new_v4()),
            amount: Amount::from_units(amount_units),
            currency: Currency::from_str("USD").expect("currency"),
            brand_code: 1,
            settlement_rail_id: 3,
        }
    }

    #[tokio::test]
    async fn settle_returns_tx_signature_as_proof() {
        let adapter = SolanaWalletAdapter::in_memory_mock();
        let context = sample_context(100);
        let hold_ref = context.hold_id.0.to_string();

        let receipt = adapter.settle(context).await.expect("settled");

        assert_eq!(receipt.rail, FundingType::SolanaWallet);
        assert_eq!(receipt.proof, format!("SOL-MEM-{hold_ref}"));
    }

    #[tokio::test]
    async fn rejects_non_positive_amount() {
        let adapter = SolanaWalletAdapter::in_memory_mock();
        let result = adapter.settle(sample_context(0)).await;

        assert_eq!(
            result,
            Err(LiquidityError::InsufficientFunds {
                rail: FundingType::SolanaWallet
            })
        );
    }

    #[tokio::test]
    async fn engine_settles_via_solana_adapter() {
        let engine = SettlementEngine::new([
            Arc::new(SolanaWalletAdapter::in_memory_mock()) as Arc<dyn SettlementAdapter>,
        ]);

        let receipt = engine
            .settle(FundingType::SolanaWallet, sample_context(25))
            .await
            .expect("settled");

        assert_eq!(receipt.rail, FundingType::SolanaWallet);
        assert!(receipt.proof.starts_with("SOL-MEM-"));
    }
}
