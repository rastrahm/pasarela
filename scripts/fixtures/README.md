> **Documentation / Documentación:** [Español (es)](README-es.md) · [English (en)](README-en.md)
>
# Fixtures canónicos — checkout API pública

> Fase 6.8 · Resuelve [DT-P1-01](../Doc/Deuda-Tecnica.md#dt-p1-01--fixtures-json-divergentes-gateway--oracle--frontend)

Payloads JSON para `POST /api/v1/checkout` (Gateway). **Fuente única** referenciada por:

- Tests Rust (`crates/api-gateway/tests/common/fixtures.rs`)
- Tests frontend (`frontend/src/schemas/gateway.test.ts`)
- Runbook / benches manuales ([Doc/Runbook-Desarrollo.md](../Doc/Runbook-Desarrollo.md))
- Playwright E2E (helpers)

## Archivos

| Archivo | Riel | Uso |
|---------|------|-----|
| `checkout-bank.json` | `traditional_bank` | Default curl / cross-service |
| `checkout-binance.json` | `binance_cex` | E2E riel Binance |
| `checkout-solana.json` | `solana_wallet` | E2E riel Solana |
| `checkout-no-rail.json` | (default comercio) | Rail Switcher elige riel |
| `checkout-invalid-amount.json` | — | Validación 422 |
| `checkout-amount-200.json` | `traditional_bank` | Conflicto idempotencia |

## Convenciones

| Campo | Valor canónico |
|-------|----------------|
| PAN | `4111111111111111` (Visa Luhn válido) |
| `expiry_month` | `"12"` |
| `expiry_year` | `"2030"` (4 dígitos — API pública Gateway/frontend) |
| `cvv` | `"123"` |
| `cardholder` | `"Demo User"` o `"Test User"` |

> **Nota Oracle wire:** el contrato interno Gateway→Oracle acepta el mismo formato; fixtures históricos en `crates/oracle-client/tests/fixtures/` usan `"2030"` alineado con la API pública.

## Uso

```bash
curl -s -X POST http://127.0.0.1:8080/api/v1/checkout \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: manual-$(date +%s)" \
  -H "Content-Type: application/json" \
  -d @scripts/fixtures/checkout-bank.json
```
