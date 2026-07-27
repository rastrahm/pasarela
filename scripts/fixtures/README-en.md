# Canonical fixtures — public checkout API

> Phase 6.8 · Resolves [DT-P1-01](../Doc/Deuda-Tecnica-en.md#dt-p1-01--fixtures-json-divergentes-gateway--oracle--frontend)

JSON payloads for `POST /api/v1/checkout` (Gateway). **Single source of truth** referenced by:

- Rust tests (`crates/api-gateway/tests/common/fixtures.rs`)
- Frontend tests (`frontend/src/schemas/gateway.test.ts`)
- Runbook / manual benches ([Doc/Runbook-Desarrollo-en.md](../Doc/Runbook-Desarrollo-en.md))
- Playwright E2E (helpers)

## Files

| File | Rail | Purpose |
|---------|------|-----|
| `checkout-bank.json` | `traditional_bank` | Default curl / cross-service |
| `checkout-binance.json` | `binance_cex` | Binance rail E2E |
| `checkout-solana.json` | `solana_wallet` | Solana rail E2E |
| `checkout-no-rail.json` | (merchant default) | Rail Switcher chooses rail |
| `checkout-invalid-amount.json` | — | 422 validation |
| `checkout-amount-200.json` | `traditional_bank` | Idempotency conflict |

## Conventions

| Field | Canonical value |
|-------|----------------|
| PAN | `4111111111111111` (valid Visa Luhn) |
| `expiry_month` | `"12"` |
| `expiry_year` | `"2030"` (4 digits — public Gateway/frontend API) |
| `cvv` | `"123"` |
| `cardholder` | `"Demo User"` or `"Test User"` |

> **Oracle wire note:** the internal Gateway→Oracle contract accepts the same format; historical fixtures in `crates/oracle-client/tests/fixtures/` use `"2030"` aligned with the public API.

## Usage

```bash
curl -s -X POST http://127.0.0.1:8080/api/v1/checkout \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: manual-$(date +%s)" \
  -H "Content-Type: application/json" \
  -d @scripts/fixtures/checkout-bank.json
```
