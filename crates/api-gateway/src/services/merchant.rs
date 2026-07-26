//! Registro de comercios y API keys (D12).

use std::collections::HashMap;
use std::sync::RwLock;

use domain::MerchantId;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::GatewayError;

/// Modo de operación de la clave (`sk_test_` vs `sk_live_`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyMode {
    Test,
    Live,
}

/// Comercio autenticado tras validar la API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedMerchant {
    pub merchant_id: MerchantId,
    pub mode: ApiKeyMode,
}

/// Catálogo de API keys (hash) → comercio.
#[derive(Default)]
pub struct MerchantRegistry {
    by_api_key_hash: RwLock<HashMap<String, AuthenticatedMerchant>>,
}

impl MerchantRegistry {
    /// Registra un comercio a partir del hash de su API key (PostgreSQL).
    pub fn register_hash(
        &self,
        api_key_hash: impl Into<String>,
        merchant_id: MerchantId,
        mode: ApiKeyMode,
    ) {
        if let Ok(mut map) = self.by_api_key_hash.write() {
            map.insert(
                api_key_hash.into(),
                AuthenticatedMerchant {
                    merchant_id,
                    mode,
                },
            );
        }
    }

    /// Registra un comercio con su API key en texto plano (dev / bootstrap in-memory).
    pub fn register(&self, api_key: impl Into<String>, merchant_id: MerchantId, mode: ApiKeyMode) {
        let api_key = api_key.into();
        self.register_hash(hash_api_key(&api_key), merchant_id, mode);
    }

    /// Resuelve un comercio a partir de la API key presentada.
    pub fn authenticate(&self, api_key: &str) -> Result<AuthenticatedMerchant, GatewayError> {
        validate_api_key_format(api_key)?;

        self.by_api_key_hash
            .read()
            .ok()
            .and_then(|map| map.get(&hash_api_key(api_key)).copied())
            .ok_or(GatewayError::Unauthorized)
    }

    /// Helper para tests con un único comercio.
    pub fn single(api_key: impl Into<String>, merchant_id: MerchantId) -> Self {
        let registry = Self::default();
        registry.register(api_key, merchant_id, ApiKeyMode::Test);
        registry
    }
}

/// Entrada de API key cargada desde configuración.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantApiKeyEntry {
    pub api_key: String,
    pub merchant_id: MerchantId,
    pub mode: ApiKeyMode,
}

/// Errores al construir el registro de comercios.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MerchantRegistryError {
    #[error("debe configurarse GATEWAY_MERCHANT_API_KEYS o GATEWAY_TEST_API_KEY")]
    MissingMerchantKeys,

    #[error("formato inválido en GATEWAY_MERCHANT_API_KEYS")]
    InvalidMerchantKeysFormat,

    #[error("API key de comercio con formato inválido")]
    InvalidApiKeyFormat,

    #[error("merchant_id inválido: {0}")]
    InvalidMerchantId(String),
}

impl MerchantRegistryError {
    /// Indica si el arranque del Gateway debe abortar.
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::MissingMerchantKeys)
    }
}

/// Parsea `sk_test_xxx:uuid,sk_live_yyy:uuid`.
pub fn parse_merchant_api_keys(raw: &str) -> Result<Vec<MerchantApiKeyEntry>, MerchantRegistryError> {
    let mut entries = Vec::new();

    for segment in raw.split(',').map(str::trim).filter(|part| !part.is_empty()) {
        let (api_key, merchant_raw) = segment
            .split_once(':')
            .ok_or(MerchantRegistryError::InvalidMerchantKeysFormat)?;

        let api_key = api_key.trim().to_string();
        let mode = api_key_mode(&api_key)?;
        let merchant_id = Uuid::parse_str(merchant_raw.trim())
            .map(MerchantId::new)
            .map_err(|_| MerchantRegistryError::InvalidMerchantId(merchant_raw.trim().to_string()))?;

        entries.push(MerchantApiKeyEntry {
            api_key,
            merchant_id,
            mode,
        });
    }

    if entries.is_empty() {
        return Err(MerchantRegistryError::InvalidMerchantKeysFormat);
    }

    Ok(entries)
}

fn api_key_mode(api_key: &str) -> Result<ApiKeyMode, MerchantRegistryError> {
    validate_api_key_format(api_key).map_err(|_| MerchantRegistryError::InvalidApiKeyFormat)?;
    if api_key.starts_with("sk_test_") {
        Ok(ApiKeyMode::Test)
    } else {
        Ok(ApiKeyMode::Live)
    }
}

fn validate_api_key_format(api_key: &str) -> Result<(), GatewayError> {
    const MIN_SUFFIX_LEN: usize = 8;

    let suffix = api_key
        .strip_prefix("sk_test_")
        .or_else(|| api_key.strip_prefix("sk_live_"));

    match suffix {
        Some(value) if value.len() >= MIN_SUFFIX_LEN && value.bytes().all(is_api_key_char) => Ok(()),
        _ => Err(GatewayError::Unauthorized),
    }
}

fn is_api_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// Calcula SHA-256 hex de una API key para persistencia segura.
pub fn hash_api_key(api_key: &str) -> String {
    let digest = Sha256::digest(api_key.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_merchant_api_keys_from_env_format() {
        let entries = parse_merchant_api_keys(
            "sk_test_demo12345678:550e8400-e29b-41d4-a716-446655440000,sk_live_prod12345678:6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        )
        .expect("entries");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].mode, ApiKeyMode::Test);
        assert_eq!(entries[1].mode, ApiKeyMode::Live);
    }

    #[test]
    fn rejects_invalid_api_key_prefix() {
        let registry = MerchantRegistry::single("sk_test_validkey1", MerchantId::new(Uuid::new_v4()));
        assert!(registry.authenticate("sk_prod_invalid1").is_err());
    }

    #[test]
    fn authenticates_registered_merchant() {
        let merchant_id = MerchantId::new(Uuid::new_v4());
        let registry = MerchantRegistry::single("sk_test_validkey1", merchant_id);
        let auth = registry
            .authenticate("sk_test_validkey1")
            .expect("authenticated");
        assert_eq!(auth.merchant_id, merchant_id);
        assert_eq!(auth.mode, ApiKeyMode::Test);
    }

    #[test]
    fn hash_api_key_is_deterministic() {
        assert_eq!(
            hash_api_key("sk_test_validkey1"),
            hash_api_key("sk_test_validkey1")
        );
        assert_ne!(
            hash_api_key("sk_test_validkey1"),
            hash_api_key("sk_test_validkey2")
        );
    }
}
