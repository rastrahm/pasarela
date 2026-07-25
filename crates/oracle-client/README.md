# oracle-client

Cliente HTTP tipado hacia el **Oracle de Autorización** (contrato API v1).

> Consumidor previsto: **API Gateway** (`pasarela/`). El frontend y otros servicios no deben invocar al Oracle directamente.

## Contrato

- DTOs: `AuthorizeRequest`, `AuthorizeResponse`, `ReleaseHoldRequest`, `ReleaseHoldResponse`, `HealthResponse`, `ErrorResponse`
- OpenAPI: [oracle/openapi/v1.yaml](../../oracle/openapi/v1.yaml)
- Doc humana: [oracle/docs/API-v1.md](../../oracle/docs/API-v1.md)

## Uso

```rust
use oracle_client::{HttpOracleClient, OracleClient, AuthorizeRequest, RequestOptions};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), oracle_client::OracleClientError> {
    let client = HttpOracleClient::new(
        "http://127.0.0.1:8081".to_string(),
        std::env::var("ORACLE_API_KEY").expect("ORACLE_API_KEY"),
        5,
    )?;

    client.health().await?;

    let request = AuthorizeRequest {
        gateway_request_id: Uuid::new_v4(),
        card: /* ... */,
        amount: 100.0,
        currency: "USD".into(),
        funding_type: oracle_client::FundingType::TraditionalBank,
    };

    let response = client
        .authorize(
            request,
            RequestOptions {
                caller_ip: Some("127.0.0.1".into()),
            },
        )
        .await?;

    println!("hold_id={}", response.hold_id);
    Ok(())
}
```

## Errores

`OracleClientError` mapea códigos HTTP y `error_code` del Oracle:

| Variante | HTTP típico |
|----------|-------------|
| `Unauthorized` | 401 |
| `Forbidden` | 403 |
| `TooManyRequests` | 429 |
| `InvalidCard` | 422 |
| `InsufficientFunds` / `FraudDeclined` | 402 |
| `RailUnavailable` | 503 |
| `NotFound` | 404 |
| `Conflict` | 409 |

## Tests

```bash
# Unitarios + mock HTTP
cargo test -p oracle-client

# Cross-service Gateway ↔ Oracle (requiere PostgreSQL)
cd oracle && ORACLE_DATABASE_URL='postgres://...' cargo test --test gateway_contract -- --test-threads=1
```
