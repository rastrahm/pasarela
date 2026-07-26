//! Configuración del adaptador Solana on-chain.

use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use thiserror::Error;

/// Parámetros RPC, programa Anchor y cuentas SPL (D10: commitment `finalized`).
pub struct SolanaSettlementConfig {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub token_mint: Pubkey,
    /// Comercio destino — MVP: env fijo; Fase 4.12 mapeará `MerchantId` → pubkey.
    pub merchant_pubkey: Pubkey,
    pub token_decimals: u8,
    pub confirm_timeout_secs: u64,
}

impl std::fmt::Debug for SolanaSettlementConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolanaSettlementConfig")
            .field("rpc_url", &self.rpc_url)
            .field("program_id", &self.program_id)
            .field("payer", &self.payer.pubkey())
            .field("token_mint", &self.token_mint)
            .field("merchant_pubkey", &self.merchant_pubkey)
            .field("token_decimals", &self.token_decimals)
            .field("confirm_timeout_secs", &self.confirm_timeout_secs)
            .finish()
    }
}

impl SolanaSettlementConfig {
    /// Carga configuración desde variables de entorno del monorepo.
    pub fn from_env() -> Result<Self, SolanaConfigError> {
        let rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

        let program_id = parse_pubkey(
            &env::var("PAYMENT_SETTLEMENT_PROGRAM_ID")
                .unwrap_or_else(|_| "4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B".to_string()),
        )?;

        let token_mint = parse_pubkey(
            &env::var("SOLANA_TOKEN_MINT").unwrap_or_else(|_| {
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
            }),
        )?;

        let merchant_pubkey = parse_pubkey(
            &env::var("SOLANA_MERCHANT_PUBKEY")
                .map_err(|_| SolanaConfigError::MissingMerchantPubkey)?,
        )?;

        let payer = load_payer_keypair()?;

        let token_decimals = env::var("SOLANA_TOKEN_DECIMALS")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .map_err(|_| SolanaConfigError::InvalidTokenDecimals)?;

        let confirm_timeout_secs = env::var("SOLANA_CONFIRM_TIMEOUT_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .map_err(|_| SolanaConfigError::InvalidConfirmTimeout)?;

        Ok(Self {
            rpc_url,
            program_id,
            payer,
            token_mint,
            merchant_pubkey,
            token_decimals,
            confirm_timeout_secs,
        })
    }
}

fn parse_pubkey(value: &str) -> Result<Pubkey, SolanaConfigError> {
    Pubkey::from_str(value.trim()).map_err(|_| SolanaConfigError::InvalidPubkey)
}

fn load_payer_keypair() -> Result<Keypair, SolanaConfigError> {
    if let Ok(path) = env::var("SOLANA_PAYER_KEYPAIR_PATH") {
        let bytes = fs::read_to_string(Path::new(&path))
            .map_err(|_| SolanaConfigError::InvalidPayerKeypair)?;
        let parsed: Vec<u8> = serde_json::from_str(&bytes)
            .map_err(|_| SolanaConfigError::InvalidPayerKeypair)?;
        return Keypair::try_from(parsed.as_slice())
            .map_err(|_| SolanaConfigError::InvalidPayerKeypair);
    }

    if let Ok(base58) = env::var("SOLANA_PAYER_KEYPAIR_BASE58") {
        let bytes = bs58::decode(base58.trim())
            .into_vec()
            .map_err(|_| SolanaConfigError::InvalidPayerKeypair)?;
        return Keypair::try_from(bytes.as_slice())
            .map_err(|_| SolanaConfigError::InvalidPayerKeypair);
    }

    Err(SolanaConfigError::MissingPayerKeypair)
}

/// Convierte un monto de dominio a base units SPL según decimales del mint.
pub fn amount_to_base_units(
    amount: domain::Amount,
    decimals: u8,
) -> Result<u64, domain::LiquidityError> {
    use domain::{FundingType, LiquidityError};

    if !amount.is_positive() {
        return Err(LiquidityError::InsufficientFunds {
            rail: FundingType::SolanaWallet,
        });
    }

    let factor = Decimal::from(10u64.pow(decimals as u32));
    let scaled = amount.value() * factor;
    let truncated = scaled.trunc();

    if truncated != scaled {
        return Err(LiquidityError::SettlementFailed);
    }

    truncated
        .try_into()
        .map(|value: u64| value)
        .map_err(|_| LiquidityError::SettlementFailed)
}

/// Errores al cargar configuración Solana.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SolanaConfigError {
    #[error("SOLANA_MERCHANT_PUBKEY es obligatoria")]
    MissingMerchantPubkey,

    #[error("SOLANA_PAYER_KEYPAIR_PATH o SOLANA_PAYER_KEYPAIR_BASE58 es obligatoria")]
    MissingPayerKeypair,

    #[error("pubkey inválida")]
    InvalidPubkey,

    #[error("keypair del pagador inválida")]
    InvalidPayerKeypair,

    #[error("SOLANA_TOKEN_DECIMALS inválido")]
    InvalidTokenDecimals,

    #[error("SOLANA_CONFIRM_TIMEOUT_SECS inválido")]
    InvalidConfirmTimeout,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::Amount;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn amount_to_base_units_scales_with_decimals() {
        let amount = Amount::new(Decimal::from_str("1.5").expect("decimal"));
        assert_eq!(amount_to_base_units(amount, 6).expect("ok"), 1_500_000);
    }

    #[test]
    fn amount_to_base_units_rejects_fractional_base_unit() {
        let amount = Amount::new(Decimal::from_str("1.0000005").expect("decimal"));
        assert_eq!(
            amount_to_base_units(amount, 6),
            Err(domain::LiquidityError::SettlementFailed)
        );
    }
}
