# Oracle de Autorización

Microservicio **independiente** de la pasarela. Simula la red procesadora (Visa/Mastercard), valida tarjetas y gestiona holds de fondos por riel.

> Solo el **API Gateway** puede invocar este servicio, vía red interna y `X-API-KEY`.

## Requisitos

- Rust 1.75+
- Variables de entorno (ver `.env.example`)

## Configuración

```bash
cp .env.example .env
# Editar ORACLE_API_KEY, ORACLE_DATABASE_URL y saldos por riel
```

### PostgreSQL (localhost)

1. Crear la base de datos:

```bash
chmod +x scripts/setup-db.sh
./scripts/setup-db.sh oracle
```

2. Ajustar `ORACLE_DATABASE_URL` en `.env`:

```
ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle
```

Las migraciones se aplican automáticamente al iniciar el servicio (`sqlx migrate`).

### Consulta de fondos por riel (Plan 2.6, UC-04)

| Riel | Adapter | Fuente |
|------|---------|--------|
| `traditional_bank` | `ConfigTraditionalBankProvider` | `TRADITIONAL_BANK_BALANCE` |
| `binance_cex` | `HttpBinanceCexProvider` | `GET /internal/v1/spot/balance` en `binance-sim/` |
| `solana_wallet` | `RpcSolanaProvider` | JSON-RPC `getTokenAccountsByOwner` |

Variables adicionales en `.env`:

| Variable | Descripción | Default |
|----------|-------------|---------|
| `BINANCE_CEX_BASE_URL` | URL del simulador Binance | `http://127.0.0.1:8083` |
| `BINANCE_CEX_API_KEY` | Clave compartida con `binance-sim/` | obligatoria |
| `SOLANA_RPC_URL` | Endpoint RPC Solana | `http://127.0.0.1:8899` |
| `SOLANA_WALLET_PUBKEY` | Wallet a consultar | obligatoria |
| `SOLANA_TOKEN_MINT` | Mint SPL (USDC) | mainnet USDC |

Los saldos `BINANCE_CEX_BALANCE` / `SOLANA_WALLET_BALANCE` se usan en tests vía `MockRailBalanceProvider`. En producción el saldo viene del HTTP/RPC.

Levantar el simulador Binance:

```bash
cd ../binance-sim && cargo run
```

Si el riel no responde (timeout/RPC caído) → `503 RAIL_UNAVAILABLE` (fail closed).

### Saldos de respaldo (tests y banco ficticio)

| Variable | Descripción | Default |
|----------|-------------|---------|
| `TRADITIONAL_BANK_BALANCE` | Saldo banco ficticio | `10000` |
| `BINANCE_CEX_BALANCE` | Mock/tests Binance | `5000` |
| `SOLANA_WALLET_BALANCE` | Mock/tests Solana | `2500` |
| `BINANCE_SPREAD_BUFFER_PCT` | Spread buffer hold Binance (D4) | `0.02` |

Los holds activos se restan del saldo disponible al evaluar fondos.

## Desarrollo

```bash
cd oracle
cargo run
```

Healthcheck: `GET http://localhost:8081/health`

## Endpoints internos (requieren autenticación)

| Método | Ruta | Descripción |
|--------|------|-------------|
| `POST` | `/internal/v1/authorize` | Validar tarjeta + evaluar fondos + crear hold |
| `POST` | `/internal/v1/hold/release` | Liberar hold si falla el settlement |
| `GET` | `/health` | Healthcheck (sin auth) |

Contrato completo: [docs/API-v1.md](docs/API-v1.md) · OpenAPI: [openapi/v1.yaml](openapi/v1.yaml)

## Seguridad

- Red privada: no exponer a Internet
- Header `X-API-KEY` obligatorio en `/internal/v1/*`
- Allowlist de IP en `ORACLE_ALLOWED_CALLERS`
- Rate limiting: ventana deslizante de 60 s por `X-API-KEY` + IP (`ORACLE_RATE_LIMIT_PER_MINUTE`, default 100) → `429`
- PAN tokenizado en memoria; nunca persistido

## Tests

```bash
# Unitarios (sin PostgreSQL)
cargo test --lib

# Integración (requiere PostgreSQL en ORACLE_DATABASE_URL)
export ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle_test
cargo test -- --test-threads=1
```

Usar `--test-threads=1` evita deadlocks en la BD compartida entre tests de integración.

- `tests/health_integration.rs` — healthcheck público
- `tests/auth_security.rs` — rechazo sin API key / key inválida
- `tests/hold_persistence.rs` — creación, release idempotente, fondos insuficientes
- `tests/rail_adapters_integration.rs` — fail closed riel caído, Binance HTTP
- `tests/logging_no_pii.rs` — ausencia de PAN/CVV en logs y audit log
- `tests/security_fail_closed.rs` — allowlist CIDR, rate limit, fail closed sin holds (Plan 2.9)
- `tests/gateway_contract.rs` — contrato Gateway ↔ Oracle vía `oracle-client` HTTP (Plan 2.11)
- `tests/auth_security.rs` — UC-11 básico (401/403)
- `tests/rate_limit_integration.rs` — 429 por API key + IP
- `tests/antifraud_integration.rs` — fail closed antifraude

## Logging estructurado (Plan 2.8)

Eventos `tracing` con campos permitidos: `gateway_request_id`, `hold_id`, `brand`, `last_four`, `token_hash`, `amount`, `currency`, `funding_type`, `caller_ip`, `reason`.

**Nunca** se registran: PAN, CVV, nombre del titular, cuerpos HTTP ni headers sensibles.

Módulo: `src/logging/` · Detector PII: `contains_forbidden_pii()`.

## Despliegue

```bash
docker build -t oracle-authorization .
docker run --env-file .env -p 8081:8081 oracle-authorization
```

En producción, conectar el contenedor solo a la red Docker/VPC del Gateway.

## Relación con la pasarela

El Gateway consume este servicio mediante el crate [`crates/oracle-client`](../crates/oracle-client/) (DTOs + cliente HTTP). No hay dependencia Cargo directa entre proyectos.

## Servicio antifraude (UC-12)

El Oracle consulta `antifraud/` **después de validar la tarjeta** y **antes de crear el hold**:

```
UC-11 (auth) → UC-03 (Luhn) → UC-12 (antifraude) → UC-04 (fondos + hold)
```

Variables en `.env`:

| Variable | Descripción |
|----------|-------------|
| `ANTIFRAUD_BASE_URL` | URL del servicio (ej. `http://127.0.0.1:8082`) |
| `ANTIFRAUD_API_KEY` | Clave compartida con antifraude |
| `ANTIFRAUD_TIMEOUT_SECS` | Timeout de scoring (default: `ORACLE_RAIL_TIMEOUT_SECS`) |

Levantar el antifraude en otra terminal:

```bash
cd ../antifraud && cargo run
```

Comportamiento fail closed: si antifraude no responde → `503`; si decline → `402 FRAUD_DECLINED`.
