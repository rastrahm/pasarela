# Binance Spot Simulado

Microservicio independiente que simula la API Spot de Binance para el riel **Binance CEX** (UC-04, UC-06).

> Solo el **Oracle** (consulta de saldo) y el **Settlement Engine** (débito) invocan las rutas internas. El Gateway y el frontend **no** acceden a este servicio.

## Desarrollo

```bash
cd binance-sim
cp .env.example .env
# BINANCE_CEX_API_KEY debe coincidir con oracle/.env
cargo run
```

Healthcheck: `GET http://localhost:8083/health`

Validar sincronización de secretos desde la raíz del monorepo:

```bash
./scripts/check-env.sh
```

## Endpoints internos

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/internal/v1/spot/balance?currency=USDC` | `X-API-KEY` | Saldo Spot disponible (Oracle) |
| `POST` | `/internal/v1/spot/debit` | `X-API-KEY` | Débito Spot simulado (liquidación CEX) |

Monedas soportadas: `USD`, `USDC`, `USDT`.

## Variables de entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| `BINANCE_SIM_HOST` | `0.0.0.0` | Host HTTP |
| `BINANCE_SIM_PORT` | `8083` | Puerto HTTP |
| `BINANCE_CEX_API_KEY` | — | Obligatoria; header `X-API-KEY` |
| `BINANCE_CEX_BALANCE` | `5000` | Saldo por defecto si no hay mapa por moneda |
| `BINANCE_CEX_BALANCES` | — | Opcional: `USDC=5000,USDT=4800` |
| `RUST_LOG` | — | Ej. `binance_sim_service=info` |

Ver [`.env.example`](.env.example) y [Doc/Runbook-Desarrollo.md](../Doc/Runbook-Desarrollo.md) §4.

## Prueba manual (curl)

```bash
export API_KEY=change-me-generate-with-openssl-rand-hex-32   # valor de BINANCE_CEX_API_KEY

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

## Comportamiento

- Saldos **in-memory** — se reinician al reiniciar el proceso.
- `spot_debit` valida monto > 0, `client_order_id` no vacío y aplica `spread_buffer_pct` (decisión D4, alineado con Oracle).
- Fondos insuficientes → `402` con `INSUFFICIENT_FUNDS`.

## Arquitectura (Axum)

```
main.rs → build_app (lib.rs) → routes/mod.rs
                                    ├─ GET  /health              (público)
                                    └─ /internal/v1/spot/*       (middleware auth/mod.rs)
```

Misma estructura que `antifraud/`: router público + rutas internas protegidas con `X-API-KEY`.

## Tests

```bash
cargo test
cargo test -p binance-sim-service --test spot_balance_integration
cargo test -p binance-sim-service --test spot_debit_integration
```

## Referencias

- [Doc/Arquitectura.md](../Doc/Arquitectura.md) — riel Binance CEX
- [oracle/README.md](../oracle/README.md) — consumidor de balance Spot
- [crates/settlement-adapters/README.md](../crates/settlement-adapters/README.md) — adapter de liquidación CEX
