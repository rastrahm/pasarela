# api-gateway

API Gateway y orquestador de checkout — **Fase 4 ✅** (acta: [Doc/Acta-Cierre-Fase-4.md](../../Doc/Acta-Cierre-Fase-4.md)).

> Expone la API pública del comercio. Solo el Gateway invoca al Oracle; el frontend nunca accede al Oracle directamente.

## Endpoints

| Método | Ruta | Auth | Estado |
|--------|------|------|--------|
| `GET` | `/health` | — | ✅ |
| `POST` | `/api/v1/checkout` | `Authorization: Bearer sk_*` + `Idempotency-Key` | ✅ |
| `GET` | `/api/v1/transactions/{id}` | `Authorization: Bearer sk_*` | ✅ |

## Arranque

```bash
cp .env.example .env
# ORACLE_API_KEY debe coincidir con oracle/.env
# GATEWAY_TEST_API_KEY + GATEWAY_DEFAULT_MERCHANT_ID para auth comercio

cargo run -p api-gateway
curl http://127.0.0.1:8080/health
```

## Prueba manual (curl / Postman)

Con Oracle y Gateway en marcha:

```bash
export GATEWAY=http://127.0.0.1:8080
export API_KEY=sk_test_change_me_32chars_min   # valor de GATEWAY_TEST_API_KEY

# Health
curl -s "$GATEWAY/health" | jq

# Checkout (riel banco)
curl -s -X POST "$GATEWAY/api/v1/checkout" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: manual-$(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 100.0,
    "currency": "USD",
    "funding_type": "traditional_bank",
    "card": {
      "pan": "4111111111111111",
      "expiry_month": "12",
      "expiry_year": "2030",
      "cvv": "123",
      "cardholder": "Demo User"
    }
  }' | jq

# Consultar transacción (sustituir TX_ID)
curl -s "$GATEWAY/api/v1/transactions/TX_ID" \
  -H "Authorization: Bearer $API_KEY" | jq
```

Rieles disponibles en checkout: `traditional_bank`, `binance_cex`, `solana_wallet`.

## Tests

```bash
# Unitarios + integración (mock Oracle + mock rieles)
cargo test -p api-gateway

# Solo gate E2E Fase 4 (paso 4.14)
cargo test -p api-gateway --test e2e_integration

# Cross-service Fase 6.2 (Oracle real + PostgreSQL)
export ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle_test
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# Rendimiento Fase 6.5 (mock Oracle + stub settlement)
cargo test -p api-gateway --test performance_integration
```

Estructura de tests de integración:

| Archivo | Alcance |
|---------|---------|
| `e2e_integration.rs` | Gate Fase 4 — flujo completo (mock Oracle) |
| `cross_service_integration.rs` | Fase 6.2 — Gateway + Oracle real (PostgreSQL) |
| `performance_integration.rs` | Fase 6.5 — latencia checkout stub p99 ≤ 500 ms |
| `security_integration.rs` | Fase 6.4 — IDOR, PCI idempotency |
| `checkout_integration.rs` | Checkout + auth |
| `rail_oracle_integration.rs` | Rail Switcher + Oracle |
| `hold_release_integration.rs` | Liberación de hold UC-04 |
| `idempotency_integration.rs` | Idempotency-Key D9 |
| `error_mapping_integration.rs` | Códigos HTTP §6.2 |

## Dependencias internas

- `domain` — tipos compartidos
- `oracle-client` — cliente HTTP hacia Oracle
- `rail-switcher` — selección de riel + fallback D3
- `settlement-adapters` — liquidación por riel (stub in-memory en dev)
