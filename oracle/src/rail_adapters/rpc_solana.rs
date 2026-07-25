//! Consulta de balance SPL vía JSON-RPC Solana (`getTokenAccountsByOwner`).

use std::time::Duration;

use std::str::FromStr;

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

use crate::funds::FundingType;

use super::client::RailBalanceProvider;
use super::error::RailAdapterError;

/// Proveedor de saldo on-chain vía RPC Solana.
#[derive(Debug, Clone)]
pub struct RpcSolanaProvider {
    http: Client,
    rpc_url: String,
    wallet_pubkey: String,
    token_mint: String,
}

impl RpcSolanaProvider {
    /// Crea el cliente RPC con wallet, mint SPL y timeout.
    pub fn new(
        rpc_url: String,
        wallet_pubkey: String,
        token_mint: String,
        timeout_secs: u64,
    ) -> Result<Self, RailAdapterError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| RailAdapterError::Unavailable(err.to_string()))?;

        Ok(Self {
            http,
            rpc_url: rpc_url.trim_end_matches('/').to_string(),
            wallet_pubkey,
            token_mint,
        })
    }
}

#[async_trait]
impl RailBalanceProvider for RpcSolanaProvider {
    async fn fetch_balance(
        &self,
        funding_type: FundingType,
        currency: &str,
    ) -> Result<Decimal, RailAdapterError> {
        if funding_type != FundingType::SolanaWallet {
            return Err(RailAdapterError::Unavailable(format!(
                "RpcSolanaProvider no atiende {funding_type:?}"
            )));
        }

        validate_solana_currency(currency)?;

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountsByOwner",
            "params": [
                self.wallet_pubkey,
                { "mint": self.token_mint },
                { "encoding": "jsonParsed" }
            ]
        });

        let response = self
            .http
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(map_transport_error)?;

        if !response.status().is_success() {
            return Err(RailAdapterError::Unavailable(format!(
                "status {}",
                response.status()
            )));
        }

        let body: RpcEnvelope = response
            .json()
            .await
            .map_err(|err| RailAdapterError::InvalidResponse(err.to_string()))?;

        if let Some(error) = body.error {
            return Err(RailAdapterError::Unavailable(format!(
                "rpc error {}: {}",
                error.code, error.message
            )));
        }

        let result = body
            .result
            .ok_or_else(|| RailAdapterError::InvalidResponse("sin result".into()))?;

        let total = sum_token_balances(&result.value)?;
        Ok(total)
    }
}

fn validate_solana_currency(currency: &str) -> Result<(), RailAdapterError> {
    let normalized = currency.to_ascii_uppercase();
    if normalized == "USD" || normalized == "USDC" {
        Ok(())
    } else {
        Err(RailAdapterError::UnsupportedCurrency(currency.to_string()))
    }
}

fn sum_token_balances(accounts: &[TokenAccountEntry]) -> Result<Decimal, RailAdapterError> {
    let mut total = Decimal::ZERO;

    for entry in accounts {
        let token_amount = &entry.account.data.parsed.info.token_amount;
        let amount = if let Some(raw) = &token_amount.ui_amount_string {
            raw.parse::<Decimal>()
                .map_err(|err| RailAdapterError::InvalidResponse(err.to_string()))?
        } else if let Some(raw) = token_amount.ui_amount {
            Decimal::from_str(&raw.to_string())
                .map_err(|err| RailAdapterError::InvalidResponse(err.to_string()))?
        } else {
            return Err(RailAdapterError::InvalidResponse(
                "tokenAmount ausente en cuenta SPL".into(),
            ));
        };

        total = total
            .checked_add(amount)
            .ok_or_else(|| RailAdapterError::InvalidResponse("overflow de saldo SPL".into()))?;
    }

    Ok(total)
}

fn map_transport_error(err: reqwest::Error) -> RailAdapterError {
    if err.is_timeout() {
        RailAdapterError::Unavailable("timeout".to_string())
    } else {
        RailAdapterError::Unavailable(err.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct RpcEnvelope {
    result: Option<TokenAccountsResult>,
    error: Option<RpcErrorBody>,
}

#[derive(Debug, Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TokenAccountsResult {
    value: Vec<TokenAccountEntry>,
}

#[derive(Debug, Deserialize)]
struct TokenAccountEntry {
    account: TokenAccountData,
}

#[derive(Debug, Deserialize)]
struct TokenAccountData {
    data: TokenParsedData,
}

#[derive(Debug, Deserialize)]
struct TokenParsedData {
    parsed: TokenParsedInfo,
}

#[derive(Debug, Deserialize)]
struct TokenParsedInfo {
    info: TokenBalanceInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenBalanceInfo {
    token_amount: TokenAmountFields,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenAmountFields {
    ui_amount_string: Option<String>,
    ui_amount: Option<f64>,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::Value;
    use tokio::net::TcpListener;

    use super::*;

    fn sample_rpc_response(amount: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "value": [{
                    "account": {
                        "data": {
                            "parsed": {
                                "info": {
                                    "tokenAmount": {
                                        "uiAmountString": amount
                                    }
                                }
                            }
                        }
                    }
                }]
            }
        })
    }

    async fn spawn_mock_rpc(response: Value) -> String {
        let app = Router::new().route(
            "/",
            post(move || {
                let response = response.clone();
                async move { Json(response) }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn rpc_client_parses_spl_balance() {
        let rpc_url = spawn_mock_rpc(sample_rpc_response("2500.50")).await;
        let provider = RpcSolanaProvider::new(
            rpc_url,
            "DemoWallet1111111111111111111111111111111".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            2,
        )
        .expect("client");

        let balance = provider
            .fetch_balance(FundingType::SolanaWallet, "USDC")
            .await
            .expect("balance");

        assert_eq!(balance, Decimal::from_str("2500.50").expect("decimal"));
    }

    #[tokio::test]
    async fn rpc_client_returns_zero_for_empty_accounts() {
        let rpc_url = spawn_mock_rpc(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "value": [] }
        }))
        .await;

        let provider = RpcSolanaProvider::new(
            rpc_url,
            "DemoWallet1111111111111111111111111111111".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            2,
        )
        .expect("client");

        let balance = provider
            .fetch_balance(FundingType::SolanaWallet, "USDC")
            .await
            .expect("balance");

        assert_eq!(balance, Decimal::ZERO);
    }
}
