//! Cliente RPC Solana — invoca `process_payment` y espera `finalized` (D10).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use thiserror::Error;

use super::config::SolanaSettlementConfig;
use super::instruction::{build_process_payment_instruction, derive_process_payment_accounts};

/// Solicitud de liquidación on-chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPaymentRequest {
    pub amount_base_units: u64,
    pub brand_code: u8,
    pub settlement_rail_id: u64,
    pub client_reference: String,
}

/// Resultado con firma de transacción confirmada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPaymentResult {
    pub signature: String,
}

/// Cliente de liquidación Solana — RPC real o mock en memoria.
#[async_trait]
pub trait SolanaSettlementClient: Send + Sync {
    /// Invoca `process_payment` y espera commitment **`finalized`** antes de retornar.
    async fn process_payment(
        &self,
        request: ProcessPaymentRequest,
    ) -> Result<ProcessPaymentResult, SolanaClientError>;
}

/// Cliente RPC que firma, envía y confirma con commitment `finalized`.
#[derive(Clone)]
pub struct RpcSolanaSettlementClient {
    config: Arc<SolanaSettlementConfig>,
}

impl RpcSolanaSettlementClient {
    /// Crea el cliente a partir de configuración cargada.
    pub fn new(config: SolanaSettlementConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    fn process_payment_blocking(
        &self,
        request: ProcessPaymentRequest,
    ) -> Result<ProcessPaymentResult, SolanaClientError> {
        if request.amount_base_units == 0 {
            return Err(SolanaClientError::InvalidRequest);
        }

        let accounts = derive_process_payment_accounts(
            &self.config.program_id,
            &self.config.payer.pubkey(),
            &self.config.merchant_pubkey,
            &self.config.token_mint,
        );

        let instruction = build_process_payment_instruction(
            &self.config.program_id,
            &accounts,
            request.amount_base_units,
            request.brand_code,
            request.settlement_rail_id,
        );

        let rpc = RpcClient::new_with_timeout_and_commitment(
            self.config.rpc_url.clone(),
            Duration::from_secs(self.config.confirm_timeout_secs),
            CommitmentConfig::finalized(),
        );

        let blockhash = rpc
            .get_latest_blockhash()
            .map_err(|err| SolanaClientError::Unavailable(err.to_string()))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.config.payer.pubkey()),
            &[&self.config.payer],
            blockhash,
        );

        let signature: Signature = rpc
            .send_and_confirm_transaction(&transaction)
            .map_err(map_rpc_error)?;

        Ok(ProcessPaymentResult {
            signature: signature.to_string(),
        })
    }
}

#[async_trait]
impl SolanaSettlementClient for RpcSolanaSettlementClient {
    async fn process_payment(
        &self,
        request: ProcessPaymentRequest,
    ) -> Result<ProcessPaymentResult, SolanaClientError> {
        let client = self.clone();
        tokio::task::spawn_blocking(move || client.process_payment_blocking(request))
            .await
            .map_err(|_| SolanaClientError::Unavailable("tarea RPC interrumpida".to_string()))?
    }
}

/// Mock en memoria para tests — retorna firma determinista sin RPC.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InMemorySolanaSettlementClient;

#[async_trait]
impl SolanaSettlementClient for InMemorySolanaSettlementClient {
    async fn process_payment(
        &self,
        request: ProcessPaymentRequest,
    ) -> Result<ProcessPaymentResult, SolanaClientError> {
        if request.amount_base_units == 0 {
            return Err(SolanaClientError::InvalidRequest);
        }

        Ok(ProcessPaymentResult {
            signature: format!("SOL-MEM-{}", request.client_reference),
        })
    }
}

/// Errores del cliente Solana.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SolanaClientError {
    #[error("solicitud inválida")]
    InvalidRequest,

    #[error("fondos SPL insuficientes")]
    InsufficientFunds,

    #[error("confirmación finalized no alcanzada en timeout")]
    FinalizationTimeout,

    #[error("riel no disponible: {0}")]
    Unavailable(String),
}

fn map_rpc_error(err: solana_client::client_error::ClientError) -> SolanaClientError {
    let message = err.to_string();
    if message.contains("InsufficientFunds") || message.contains("insufficient") {
        SolanaClientError::InsufficientFunds
    } else if message.contains("timeout") || message.contains("TimedOut") {
        SolanaClientError::FinalizationTimeout
    } else {
        SolanaClientError::Unavailable(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_returns_signature_prefix() {
        let client = InMemorySolanaSettlementClient;
        let result = client
            .process_payment(ProcessPaymentRequest {
                amount_base_units: 100_000,
                brand_code: 1,
                settlement_rail_id: 3,
                client_reference: "hold-abc".to_string(),
            })
            .await
            .expect("ok");

        assert_eq!(result.signature, "SOL-MEM-hold-abc");
    }

    #[tokio::test]
    async fn in_memory_rejects_zero_amount() {
        let client = InMemorySolanaSettlementClient;
        let result = client
            .process_payment(ProcessPaymentRequest {
                amount_base_units: 0,
                brand_code: 1,
                settlement_rail_id: 3,
                client_reference: "hold".to_string(),
            })
            .await;

        assert_eq!(result, Err(SolanaClientError::InvalidRequest));
    }
}
