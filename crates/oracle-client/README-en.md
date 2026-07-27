# oracle-client

Typed HTTP client for the **Authorization Oracle** (API v1 contract).

> Intended consumer: **API Gateway** (`pasarela/`). The frontend and other services must not invoke the Oracle directly.

## Contract

- DTOs: `AuthorizeRequest`, `AuthorizeResponse`, `ReleaseHoldRequest`, `ReleaseHoldResponse`, `HealthResponse`, `ErrorResponse`
- OpenAPI: [oracle/openapi/v1.yaml](../../oracle/openapi/v1.yaml)
- Human docs: [oracle/docs/API-v1-en.md](../../oracle/docs/API-v1-en.md)

## Usage

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

## Errors

`OracleClientError` maps HTTP codes and Oracle `error_code`:

| Variant | Typical HTTP |
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
# Unit tests + HTTP mock
cargo test -p oracle-client

# Cross-service Gateway ↔ Oracle (requires PostgreSQL)
cd oracle && ORACLE_DATABASE_URL='postgres://...' cargo test --test gateway_contract -- --test-threads=1
```
