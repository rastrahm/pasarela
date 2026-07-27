# Simulated Binance Spot

Standalone microservice that simulates the Binance Spot API for the **Binance CEX** rail (UC-04, UC-06).

> Only the **Oracle** (balance query) and the **Settlement Engine** (debit) invoke the internal routes. The Gateway and frontend **do not** access this service.

## Development

```bash
cd binance-sim
cp .env.example .env
# BINANCE_CEX_API_KEY must match oracle/.env
cargo run
```

Healthcheck: `GET http://localhost:8083/health`

Validate secret synchronization from the monorepo root:

```bash
./scripts/check-env.sh
```

## Internal endpoints

| Method | Route | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/internal/v1/spot/balance?currency=USDC` | `X-API-KEY` | Available Spot balance (Oracle) |
| `POST` | `/internal/v1/spot/debit` | `X-API-KEY` | Simulated Spot debit (CEX settlement) |

Supported currencies: `USD`, `USDC`, `USDT`.

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BINANCE_SIM_HOST` | `0.0.0.0` | HTTP host |
| `BINANCE_SIM_PORT` | `8083` | HTTP port |
| `BINANCE_CEX_API_KEY` | — | Required; `X-API-KEY` header |
| `BINANCE_CEX_BALANCE` | `5000` | Default balance if no per-currency map |
| `BINANCE_CEX_BALANCES` | — | Optional: `USDC=5000,USDT=4800` |
| `RUST_LOG` | — | E.g. `binance_sim_service=info` |

See [`.env.example`](.env.example) and [Doc/Runbook-Desarrollo-en.md](../Doc/Runbook-Desarrollo-en.md) §4.

## Manual test (curl)

```bash
export API_KEY=change-me-generate-with-openssl-rand-hex-32   # value of BINANCE_CEX_API_KEY

curl -s "http://127.0.0.1:8083/internal/v1/spot/balance?currency=USDC" \
  -H "X-API-KEY: $API_KEY" | jq

curl -s -X POST "http://127.0.0.1:8083/internal/v1/spot/debit" \
  -H "X-API-KEY: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": "100.0",
    "currency": "USDC",
    "client_order_id": "manual-test-001",
    "spread_buffer_pct": "0.02"
  }' | jq
```

## Behavior

- **In-memory** balances — reset on process restart.
- `spot_debit` validates amount > 0, non-empty `client_order_id`, and applies `spread_buffer_pct` (decision D4, aligned with Oracle).
- Insufficient funds → `402` with `INSUFFICIENT_FUNDS`.

## Architecture (Axum)

```
main.rs → build_app (lib.rs) → routes/mod.rs
                                    ├─ GET  /health              (public)
                                    └─ /internal/v1/spot/*       (middleware auth/mod.rs)
```

Same structure as `antifraud/`: public router + internal routes protected with `X-API-KEY`.

## Tests

```bash
cargo test
cargo test -p binance-sim-service --test spot_balance_integration
cargo test -p binance-sim-service --test spot_debit_integration
```

## References

- [Doc/Arquitectura-en.md](../Doc/Arquitectura-en.md) — Binance CEX rail
- [oracle/README-en.md](../oracle/README-en.md) — Spot balance consumer
- [crates/settlement-adapters/README-en.md](../crates/settlement-adapters/README-en.md) — CEX settlement adapter
