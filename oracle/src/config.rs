//! Carga de configuraci?n desde variables de entorno.

use std::env;
use std::net::IpAddr;

use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// Configuraci?n del servicio Oracle.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub api_key: String,
    pub allowed_callers: Vec<CallerRule>,
    pub rate_limit_per_minute: u32,
    pub rail_timeout_secs: u64,
    pub hold_ttl_secs: u64,
    pub ttl_cleanup_interval_secs: u64,
    pub traditional_bank_balance: Decimal,
    pub binance_cex_balance: Decimal,
    pub solana_wallet_balance: Decimal,
    pub binance_spread_buffer_pct: Decimal,
    pub binance_cex_base_url: String,
    pub binance_cex_api_key: String,
    pub solana_rpc_url: String,
    pub solana_wallet_pubkey: String,
    pub solana_token_mint: String,
    pub antifraud_base_url: String,
    pub antifraud_api_key: String,
    pub antifraud_timeout_secs: u64,
}

/// Regla de origen permitido: IP exacta o prefijo CIDR simplificado.
#[derive(Debug, Clone)]
pub enum CallerRule {
    Exact(IpAddr),
    Prefix { base: IpAddr, prefix_len: u8 },
}

impl AppConfig {
    /// Carga la configuraci?n desde el entorno.
    ///
    /// # Returns
    /// `AppConfig` o error si faltan variables cr?ticas.
    pub fn from_env() -> Result<Self> {
        let host = env::var("ORACLE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("ORACLE_PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse()
            .context("ORACLE_PORT debe ser un entero v?lido")?;

        let database_url = env::var("ORACLE_DATABASE_URL")
            .context("ORACLE_DATABASE_URL es obligatoria")?;

        let api_key = env::var("ORACLE_API_KEY").context("ORACLE_API_KEY es obligatoria")?;

        let allowed_raw = env::var("ORACLE_ALLOWED_CALLERS")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let allowed_callers = parse_allowed_callers(&allowed_raw)?;

        let rate_limit_per_minute = env::var("ORACLE_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .context("ORACLE_RATE_LIMIT_PER_MINUTE inv?lido")?;

        let rail_timeout_secs: u64 = env::var("ORACLE_RAIL_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("ORACLE_RAIL_TIMEOUT_SECS inv?lido")?;

        let hold_ttl_secs = env::var("ORACLE_HOLD_TTL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .context("ORACLE_HOLD_TTL_SECS inv?lido")?;

        let ttl_cleanup_interval_secs = env::var("ORACLE_TTL_CLEANUP_INTERVAL_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .context("ORACLE_TTL_CLEANUP_INTERVAL_SECS inv?lido")?;

        let traditional_bank_balance = parse_decimal_env(
            "TRADITIONAL_BANK_BALANCE",
            "10000",
        )?;
        let binance_cex_balance = parse_decimal_env("BINANCE_CEX_BALANCE", "5000")?;
        let solana_wallet_balance = parse_decimal_env("SOLANA_WALLET_BALANCE", "2500")?;
        let binance_spread_buffer_pct =
            parse_decimal_env("BINANCE_SPREAD_BUFFER_PCT", "0.02")?;

        let binance_cex_base_url = env::var("BINANCE_CEX_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8083".to_string());
        let binance_cex_api_key =
            env::var("BINANCE_CEX_API_KEY").context("BINANCE_CEX_API_KEY es obligatoria")?;

        let solana_rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());
        let solana_wallet_pubkey = env::var("SOLANA_WALLET_PUBKEY")
            .context("SOLANA_WALLET_PUBKEY es obligatoria")?;
        let solana_token_mint = env::var("SOLANA_TOKEN_MINT").unwrap_or_else(|_| {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
        });

        let antifraud_base_url = env::var("ANTIFRAUD_BASE_URL")
            .context("ANTIFRAUD_BASE_URL es obligatoria")?;
        let antifraud_api_key =
            env::var("ANTIFRAUD_API_KEY").context("ANTIFRAUD_API_KEY es obligatoria")?;
        let antifraud_timeout_secs = env::var("ANTIFRAUD_TIMEOUT_SECS")
            .unwrap_or_else(|_| rail_timeout_secs.to_string())
            .parse()
            .context("ANTIFRAUD_TIMEOUT_SECS inv?lido")?;

        Ok(Self {
            host,
            port,
            database_url,
            api_key,
            allowed_callers,
            rate_limit_per_minute,
            rail_timeout_secs,
            hold_ttl_secs,
            ttl_cleanup_interval_secs,
            traditional_bank_balance,
            binance_cex_balance,
            solana_wallet_balance,
            binance_spread_buffer_pct,
            binance_cex_base_url,
            binance_cex_api_key,
            solana_rpc_url,
            solana_wallet_pubkey,
            solana_token_mint,
            antifraud_base_url,
            antifraud_api_key,
            antifraud_timeout_secs,
        })
    }

    /// Direcci?n de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Saldo configurado para un riel (antes de holds activos).
    pub fn configured_balance(&self, funding_type: crate::funds::FundingType) -> Decimal {
        use crate::funds::FundingType;

        match funding_type {
            FundingType::TraditionalBank => self.traditional_bank_balance,
            FundingType::BinanceCex => self.binance_cex_balance,
            FundingType::SolanaWallet => self.solana_wallet_balance,
        }
    }
}

fn parse_decimal_env(key: &str, default: &str) -> Result<Decimal> {
    let raw = env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<Decimal>()
        .with_context(|| format!("{key} debe ser un decimal v?lido"))
}

/// Parsea la lista de callers permitidos separada por comas.
fn parse_allowed_callers(raw: &str) -> Result<Vec<CallerRule>> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_caller_rule)
        .collect()
}

fn parse_caller_rule(entry: &str) -> Result<CallerRule> {
    if let Some((base, prefix)) = entry.split_once('/') {
        let base_ip: IpAddr = base.parse().context("IP base inv?lida en allowlist")?;
        let prefix_len: u8 = prefix
            .parse()
            .context("longitud de prefijo CIDR inv?lida")?;
        return Ok(CallerRule::Prefix {
            base: base_ip,
            prefix_len,
        });
    }

    let ip: IpAddr = entry.parse().context("IP inv?lida en allowlist")?;
    Ok(CallerRule::Exact(ip))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exact_ip() {
        let rules = parse_allowed_callers("127.0.0.1,::1").expect("parse");
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn parse_cidr_prefix() {
        let rule = parse_caller_rule("172.17.0.0/16").expect("parse");
        assert!(matches!(rule, CallerRule::Prefix { .. }));
    }
}
