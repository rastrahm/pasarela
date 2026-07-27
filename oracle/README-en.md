# Authorization Oracle

**Standalone** microservice for the payment gateway. Simulates the processing network (Visa/Mastercard), validates cards, and manages fund holds per rail.

> Only the **API Gateway** can invoke this service, via internal network and `X-API-KEY`.

## Requirements

- Rust 1.75+
- Environment variables (see `.env.example`)

## Configuration

```bash
cp .env.example .env
# Edit ORACLE_API_KEY, ORACLE_DATABASE_URL, and balances per rail
```

### PostgreSQL (localhost)

1. Create the database:

```bash
chmod +x scripts/setup-db.sh
./scripts/setup-db.sh oracle
```

2. Adjust `ORACLE_DATABASE_URL` in `.env`:

```
ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle
```

Migrations are applied automatically on service startup (`sqlx migrate`).

### Fund query per rail (Plan 2.6, UC-04)

| Rail | Adapter | Source |
|------|---------|--------|
| `traditional_bank` | `ConfigTraditionalBankProvider` | `TRADITIONAL_BANK_BALANCE` |
| `binance_cex` | `HttpBinanceCexProvider` | `GET /internal/v1/spot/balance` on `binance-sim/` |
| `solana_wallet` | `RpcSolanaProvider` | JSON-RPC `getTokenAccountsByOwner` |

Additional variables in `.env`:

| Variable | Description | Default |
|----------|-------------|---------|
| `BINANCE_CEX_BASE_URL` | Binance simulator URL | `http://127.0.0.1:8083` |
| `BINANCE_CEX_API_KEY` | Shared key with `binance-sim/` | required |
| `SOLANA_RPC_URL` | Solana RPC endpoint | `http://127.0.0.1:8899` |
| `SOLANA_WALLET_PUBKEY` | Wallet to query | required |
| `SOLANA_TOKEN_MINT` | SPL mint (USDC) | mainnet USDC |

`BINANCE_CEX_BALANCE` / `SOLANA_WALLET_BALANCE` balances are used in tests via `MockRailBalanceProvider`. In production, balance comes from HTTP/RPC.

Start the Binance simulator:

```bash
cd ../binance-sim && cargo run
```

If the rail does not respond (timeout/down RPC) → `503 RAIL_UNAVAILABLE` (fail closed).

### Fallback balances (tests and mock bank)

| Variable | Description | Default |
|----------|-------------|---------|
| `TRADITIONAL_BANK_BALANCE` | Mock bank balance | `10000` |
| `BINANCE_CEX_BALANCE` | Binance mock/tests | `5000` |
| `SOLANA_WALLET_BALANCE` | Solana mock/tests | `2500` |
| `BINANCE_SPREAD_BUFFER_PCT` | Binance hold spread buffer (D4) | `0.02` |

Active holds are subtracted from available balance when evaluating funds.

## Development

```bash
cd oracle
cargo run
```

Healthcheck: `GET http://localhost:8081/health`

## Internal endpoints (authentication required)

| Method | Route | Description |
|--------|------|-------------|
| `POST` | `/internal/v1/authorize` | Validate card + evaluate funds + create hold |
| `POST` | `/internal/v1/hold/release` | Release hold if settlement fails |
| `GET` | `/health` | Healthcheck (no auth) |

Full contract: [docs/API-v1-en.md](docs/API-v1-en.md) · OpenAPI: [openapi/v1.yaml](openapi/v1.yaml)

## Security

- Private network: do not expose to the Internet
- Header `X-API-KEY` required on `/internal/v1/*`
- IP allowlist in `ORACLE_ALLOWED_CALLERS`
- Rate limiting: 60 s sliding window per `X-API-KEY` + IP (`ORACLE_RATE_LIMIT_PER_MINUTE`, default 100) → `429`
- PAN tokenized in memory; never persisted

## Tests

```bash
# Unit tests (no PostgreSQL)
cargo test --lib

# Integration (requires PostgreSQL in ORACLE_DATABASE_URL)
export ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle_test
cargo test -p oracle-authorization --test gateway_contract -- --test-threads=1

# Cross-service from Gateway (Phase 6.2)
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1
```

Using `--test-threads=1` avoids deadlocks on the shared DB between integration tests.

- `tests/health_integration.rs` — public healthcheck
- `tests/auth_security.rs` — rejection without API key / invalid key
- `tests/hold_persistence.rs` — creation, idempotent release, insufficient funds
- `tests/rail_adapters_integration.rs` — fail closed on down rail, Binance HTTP
- `tests/logging_no_pii.rs` — absence of PAN/CVV in logs and audit log
- `tests/security_fail_closed.rs` — CIDR allowlist, rate limit, fail closed without holds (Plan 2.9)
- `tests/gateway_contract.rs` — Gateway ↔ Oracle contract via `oracle-client` HTTP (Plan 2.11)
- `tests/auth_security.rs` — basic UC-11 (401/403)
- `tests/rate_limit_integration.rs` — 429 by API key + IP
- `tests/antifraud_integration.rs` — antifraud fail closed

## Structured logging (Plan 2.8)

`tracing` events with allowed fields: `gateway_request_id`, `hold_id`, `brand`, `last_four`, `token_hash`, `amount`, `currency`, `funding_type`, `caller_ip`, `reason`.

**Never** logged: PAN, CVV, cardholder name, HTTP bodies, or sensitive headers.

Module: `src/logging/` · PII detector: `contains_forbidden_pii()`.

## Deployment

```bash
cp .env.example .env   # edit variables
cargo run --release
```

In production, run the binary only on internal network accessible by the Gateway (IP allowlist / VPC).

## Relationship with the payment gateway

The Gateway consumes this service via the [`crates/oracle-client`](../crates/oracle-client/) crate (DTOs + HTTP client). There is no direct Cargo dependency between projects.

## Antifraud service (UC-12)

The Oracle queries `antifraud/` **after validating the card** and **before creating the hold**:

```
UC-11 (auth) → UC-03 (Luhn) → UC-12 (antifraud) → UC-04 (funds + hold)
```

Variables in `.env`:

| Variable | Description |
|----------|-------------|
| `ANTIFRAUD_BASE_URL` | Service URL (e.g. `http://127.0.0.1:8082`) |
| `ANTIFRAUD_API_KEY` | Shared key with antifraud |
| `ANTIFRAUD_TIMEOUT_SECS` | Scoring timeout (default: `ORACLE_RAIL_TIMEOUT_SECS`) |

Start antifraud in another terminal:

```bash
cd ../antifraud && cargo run
```

Fail closed behavior: if antifraud does not respond → `503`; if decline → `402 FRAUD_DECLINED`.
