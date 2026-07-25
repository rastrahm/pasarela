//! Carga de configuración desde variables de entorno.

use std::env;
use std::net::IpAddr;

use anyhow::{Context, Result};

/// Configuración del servicio Oracle.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub allowed_callers: Vec<CallerRule>,
    pub rate_limit_per_minute: u32,
    pub rail_timeout_secs: u64,
    pub hold_ttl_secs: u64,
}

/// Regla de origen permitido: IP exacta o prefijo CIDR simplificado.
#[derive(Debug, Clone)]
pub enum CallerRule {
    Exact(IpAddr),
    Prefix { base: IpAddr, prefix_len: u8 },
}

impl AppConfig {
    /// Carga la configuración desde el entorno.
    ///
    /// # Returns
    /// `AppConfig` o error si faltan variables críticas en producción.
    pub fn from_env() -> Result<Self> {
        let host = env::var("ORACLE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("ORACLE_PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse()
            .context("ORACLE_PORT debe ser un entero válido")?;

        let api_key = env::var("ORACLE_API_KEY")
            .context("ORACLE_API_KEY es obligatoria")?;

        let allowed_raw = env::var("ORACLE_ALLOWED_CALLERS")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let allowed_callers = parse_allowed_callers(&allowed_raw)?;

        let rate_limit_per_minute = env::var("ORACLE_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .context("ORACLE_RATE_LIMIT_PER_MINUTE inválido")?;

        let rail_timeout_secs = env::var("ORACLE_RAIL_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("ORACLE_RAIL_TIMEOUT_SECS inválido")?;

        let hold_ttl_secs = env::var("ORACLE_HOLD_TTL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .context("ORACLE_HOLD_TTL_SECS inválido")?;

        Ok(Self {
            host,
            port,
            api_key,
            allowed_callers,
            rate_limit_per_minute,
            rail_timeout_secs,
            hold_ttl_secs,
        })
    }

    /// Dirección de escucha `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
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
        let base_ip: IpAddr = base.parse().context("IP base inválida en allowlist")?;
        let prefix_len: u8 = prefix
            .parse()
            .context("longitud de prefijo CIDR inválida")?;
        return Ok(CallerRule::Prefix {
            base: base_ip,
            prefix_len,
        });
    }

    let ip: IpAddr = entry.parse().context("IP inválida en allowlist")?;
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
