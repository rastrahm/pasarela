//! Proveedor compuesto que enruta por `FundingType`.

use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::config::AppConfig;
use crate::funds::FundingType;

use super::client::RailBalanceProvider;
use super::config_bank::ConfigTraditionalBankProvider;
use super::error::RailAdapterError;
use super::http_binance::HttpBinanceCexProvider;
use super::rpc_solana::RpcSolanaProvider;

/// Despacha la consulta de saldo al adapter concreto de cada riel.
pub struct CompositeRailProvider {
    bank: ConfigTraditionalBankProvider,
    binance: HttpBinanceCexProvider,
    solana: RpcSolanaProvider,
}

impl CompositeRailProvider {
    /// Construye los tres adapters desde la configuración del Oracle.
    pub fn new(config: Arc<AppConfig>) -> Result<Self, RailAdapterError> {
        Ok(Self {
            bank: ConfigTraditionalBankProvider::new(config.clone()),
            binance: HttpBinanceCexProvider::new(
                config.binance_cex_base_url.clone(),
                config.binance_cex_api_key.clone(),
                config.rail_timeout_secs,
            )?,
            solana: RpcSolanaProvider::new(
                config.solana_rpc_url.clone(),
                config.solana_wallet_pubkey.clone(),
                config.solana_token_mint.clone(),
                config.rail_timeout_secs,
            )?,
        })
    }
}

#[async_trait]
impl RailBalanceProvider for CompositeRailProvider {
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError> {
        match funding_type {
            FundingType::TraditionalBank => {
                self.bank.fetch_balance(funding_type, currency).await
            }
            FundingType::BinanceCex => {
                self.binance.fetch_balance(funding_type, currency).await
            }
            FundingType::SolanaWallet => {
                self.solana.fetch_balance(funding_type, currency).await
            }
        }
    }
}
