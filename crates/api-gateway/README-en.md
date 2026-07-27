# api-gateway

API Gateway and checkout orchestrator — **Phase 4 ✅** (record: [Doc/Acta-Cierre-Fase-4-en.md](../../Doc/Acta-Cierre-Fase-4-en.md)).

> Exposes the public merchant API. Only the Gateway invokes the Oracle; the frontend never accesses the Oracle directly.

## Endpoints

| Method | Route | Auth | Status |
|--------|------|------|--------|
| `GET` | `/health` | — | ✅ |
| `POST` | `/api/v1/checkout` | `Authorization: Bearer sk_*` + `Idempotency-Key` | ✅ |
| `GET` | `/api/v1/transactions/{id}` | `Authorization: Bearer sk_*` | ✅ |

## Startup

```bash
cp .env.example .env
# ORACLE_API_KEY must match oracle/.env
# GATEWAY_TEST_API_KEY + GATEWAY_DEFAULT_MERCHANT_ID for merchant auth

cd crates/api-gateway && cargo run   # recommended — loads local .env
curl http://127.0.0.1:8080/health
```

### CORS (development / E2E / staging)

The Gateway allows origins `http://127.0.0.1:5173` and `http://localhost:5173` for Playwright E2E. Additional origins via `GATEWAY_CORS_ORIGINS` (comma-separated). In staging with Caddy same-origin, CORS is usually not needed.

## Manual test (curl / Postman)

With Oracle and Gateway running:

```bash
export GATEWAY=http://127.0.0.1:8080
export API_KEY=sk_test_change_me_32chars_min   # value of GATEWAY_TEST_API_KEY

# Health
curl -s "$GATEWAY/health" | jq

# Checkout (bank rail)
curl -s -X POST "$GATEWAY/api/v1/checkout" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: manual-$(uuidgen)" \
  -H "Content-Type: application/json" \
  -d @../../scripts/fixtures/checkout-bank.json | jq

# Query transaction (replace TX_ID)
curl -s "$GATEWAY/api/v1/transactions/TX_ID" \
  -H "Authorization: Bearer $API_KEY" | jq
```

Available checkout rails: `traditional_bank`, `binance_cex`, `solana_wallet`.

## Tests

```bash
# Unit + integration (mock Oracle + stub settlement)
cargo test -p api-gateway

# Canonical fixture contract (scripts/fixtures/)
cargo test -p api-gateway --test contract_fixtures

# Phase 4 E2E gate only (step 4.14)
cargo test -p api-gateway --test e2e_integration

# Cross-service Phase 6.2 (real Oracle + PostgreSQL)
export ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle_test
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# Performance Phase 6.5 (mock Oracle + stub settlement)
cargo test -p api-gateway --test performance_integration
```

Integration test structure:

| File | Scope |
|---------|---------|
| `contract_fixtures.rs` | JSON fixtures → `CheckoutRequest` (Phase 6.8) |
| `e2e_integration.rs` | Phase 4 gate — full flow (mock Oracle) |
| `cross_service_integration.rs` | Phase 6.2 — Gateway + real Oracle (PostgreSQL) |
| `performance_integration.rs` | Phase 6.5 — stub checkout latency p99 ≤ 500 ms |
| `security_integration.rs` | Phase 6.4 — IDOR, PCI idempotency |
| `checkout_integration.rs` | Checkout + auth |
| `rail_oracle_integration.rs` | Rail Switcher + Oracle |
| `hold_release_integration.rs` | Hold release UC-04 |
| `idempotency_integration.rs` | Idempotency-Key D9 |
| `error_mapping_integration.rs` | HTTP codes §6.2 |

Full guide: [Doc/Pruebas-en.md](../../Doc/Pruebas-en.md) · E2E Playwright: [tests/e2e/README-en.md](../../tests/e2e/README-en.md)

## Internal dependencies

- `domain` — shared types
- `oracle-client` — HTTP client to Oracle
- `rail-switcher` — rail selection + fallback D3
- `settlement-adapters` — settlement per rail (in-memory stub in dev)
