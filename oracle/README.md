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

### Saldos simulados por riel

| Variable | Descripción | Default |
|----------|-------------|---------|
| `TRADITIONAL_BANK_BALANCE` | Saldo banco ficticio | `10000` |
| `BINANCE_CEX_BALANCE` | Saldo Binance Spot simulado | `5000` |
| `SOLANA_WALLET_BALANCE` | Saldo wallet Solana simulado | `2500` |
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

## Seguridad

- Red privada: no exponer a Internet
- Header `X-API-KEY` obligatorio en `/internal/v1/*`
- Allowlist de IP en `ORACLE_ALLOWED_CALLERS`
- Rate limiting configurable
- PAN tokenizado en memoria; nunca persistido

## Tests

```bash
# Unitarios (sin PostgreSQL)
cargo test --lib

# Integración (requiere PostgreSQL en ORACLE_DATABASE_URL)
export ORACLE_DATABASE_URL=postgres://postgres:<password>@localhost:5432/oracle_test
cargo test
```

- `tests/health_integration.rs` — healthcheck público
- `tests/auth_security.rs` — rechazo sin API key / key inválida
- `tests/hold_persistence.rs` — creación, release idempotente, fondos insuficientes

## Despliegue

```bash
docker build -t oracle-authorization .
docker run --env-file .env -p 8081:8081 oracle-authorization
```

En producción, conectar el contenedor solo a la red Docker/VPC del Gateway.

## Relación con la pasarela

El Gateway consume este servicio mediante el crate `pasarela/crates/oracle-client/` (pendiente de Fase 4). No hay dependencia Cargo directa entre proyectos.

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
