//! Autenticación de comercios — `Authorization: Bearer sk_test_...` (D12).

use axum::http::HeaderMap;

use crate::error::GatewayError;
use crate::services::merchant::{AuthenticatedMerchant, MerchantRegistry};

/// Extrae el token Bearer del header `Authorization`.
pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, GatewayError> {
    let value = headers
        .get("authorization")
        .ok_or(GatewayError::Unauthorized)?
        .to_str()
        .map_err(|_| GatewayError::Unauthorized)?
        .trim();

    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .ok_or(GatewayError::Unauthorized)
}

/// Valida la API key del comercio y devuelve su identidad.
pub fn authenticate_merchant(
    headers: &HeaderMap,
    registry: &MerchantRegistry,
) -> Result<AuthenticatedMerchant, GatewayError> {
    let api_key = extract_bearer_token(headers)?;
    registry.authenticate(&api_key)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::*;
    use crate::services::merchant::{ApiKeyMode, MerchantRegistry};
    use domain::MerchantId;
    use uuid::Uuid;

    fn registry_with_key(key: &str) -> MerchantRegistry {
        MerchantRegistry::single(key, MerchantId::new(Uuid::new_v4()))
    }

    #[test]
    fn extract_bearer_token_parses_authorization_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer sk_test_validkey1".parse().expect("header"),
        );

        assert_eq!(
            extract_bearer_token(&headers).expect("token"),
            "sk_test_validkey1"
        );
    }

    #[test]
    fn missing_authorization_returns_unauthorized() {
        let headers = HeaderMap::new();
        assert!(matches!(
            extract_bearer_token(&headers),
            Err(GatewayError::Unauthorized)
        ));
    }

    #[test]
    fn authenticate_merchant_resolves_registry_entry() {
        let key = "sk_test_validkey1";
        let registry = registry_with_key(key);
        let merchant_id = registry
            .authenticate(key)
            .expect("registered")
            .merchant_id;

        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {key}").parse().expect("header"));

        let auth = authenticate_merchant(&headers, &registry).expect("auth");
        assert_eq!(auth.merchant_id, merchant_id);
        assert_eq!(auth.mode, ApiKeyMode::Test);
    }

    #[test]
    fn authenticate_merchant_rejects_unknown_key() {
        let registry = registry_with_key("sk_test_validkey1");
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer sk_test_unknown1".parse().expect("header"),
        );

        assert!(matches!(
            authenticate_merchant(&headers, &registry),
            Err(GatewayError::Unauthorized)
        ));
    }
}
