# Runbook de desarrollo — Pasarela Multi-Rail

> Fase 6.7 · Stack local sin contenedores  
> Resuelve [DT-P0-03](./Deuda-Tecnica.md#dt-p0-03--runbook-de-stack-local-disperso) y tabla canónica de [DT-P0-04](./Deuda-Tecnica.md#dt-p0-04--variables-de-entorno-desincronizadas-entre-servicios).

## 1. Requisitos previos

| Herramienta | Versión mínima | Uso |
|-------------|----------------|-----|
| Rust | 1.75+ | Gateway, Oracle, simuladores |
| PostgreSQL | 14+ (16 en CI) | Oracle (obligatorio) |
| Node.js | 18+ (20 en CI) | Frontend, Playwright |
| pnpm | 9+ | Frontend, E2E |
| `curl`, `jq` | cualquiera | Healthchecks manuales |

Opcional según riel:

| Herramienta | Riel |
|-------------|------|
| Solana CLI + validador local | `solana_wallet` (consulta RPC en Oracle) |
| Anchor | Solo `programs/payment-settlement/` (fuera del stack checkout MVP) |

---

## 2. Configuración inicial (una vez)

### 2.1 Copiar archivos de entorno

Desde la raíz del monorepo:

```bash
cp binance-sim/.env.example   binance-sim/.env
cp antifraud/.env.example     antifraud/.env
cp oracle/.env.example        oracle/.env
cp crates/api-gateway/.env.example crates/api-gateway/.env
cp frontend/.env.example      frontend/.env.local
```

> **Importante:** `dotenvy` carga `.env` desde el **directorio de trabajo actual**. Ejecutá cada servicio Rust desde su carpeta (ver §3) o exportá las variables antes de `cargo run -p …` desde la raíz.

### 2.2 Sincronizar secretos compartidos

Generá una clave de 32 bytes hex y usala en todos los pares indicados:

```bash
openssl rand -hex 32   # ejemplo: a1b2c3...
```

| Par de variables | Archivos |
|------------------|----------|
| `ORACLE_API_KEY` | `crates/api-gateway/.env` ↔ `oracle/.env` |
| `GATEWAY_TEST_API_KEY` ↔ `VITE_GATEWAY_API_KEY` | `crates/api-gateway/.env` ↔ `frontend/.env.local` |
| `BINANCE_CEX_API_KEY` | `oracle/.env` ↔ `binance-sim/.env` |
| `ANTIFRAUD_API_KEY` | `oracle/.env` ↔ `antifraud/.env` |

Validación automática:

```bash
chmod +x scripts/check-env.sh
./scripts/check-env.sh          # solo archivos .env
./scripts/check-env.sh --live   # + healthchecks HTTP
```

### 2.3 Base de datos Oracle

```bash
cd oracle
chmod +x scripts/setup-db.sh
./scripts/setup-db.sh oracle
# Ajustar ORACLE_DATABASE_URL en oracle/.env si tu usuario/contraseña difieren
```

Las migraciones SQLx se aplican al arrancar el Oracle.

### 2.4 Gateway sin PostgreSQL (opcional)

El Gateway puede usar almacén **in-memory** si comentás o eliminás `DATABASE_URL` en `crates/api-gateway/.env`. Útil para pruebas rápidas; la persistencia entre reinicios no se conserva.

| Servicio | BD default | Obligatoria |
|----------|------------|-------------|
| Oracle | `postgres://postgres:postgres@localhost:5432/oracle` | Sí |
| Gateway | `postgres://pasarela:pasarela@127.0.0.1:5432/pasarela_gateway` | No (fallback in-memory) |

---

## 3. Orden de arranque

Dependencias entre servicios:

```mermaid
flowchart BT
  PG[(PostgreSQL)]
  BS[binance-sim :8083]
  AF[antifraud :8082]
  OR[Oracle :8081]
  GW[Gateway :8080]
  FE[Frontend :5173]

  PG --> OR
  BS --> OR
  AF --> OR
  OR --> GW
  GW --> FE
```

### Terminal 1 — PostgreSQL

Asegurate de que PostgreSQL escucha en `localhost:5432` y que existe la BD `oracle`.

### Terminal 2 — Simuladores (según riel)

**Siempre recomendado** (Oracle consulta antifraude en cada autorización):

```bash
cd antifraud
cargo run
# Health: curl -s http://127.0.0.1:8082/health
```

**Solo riel Binance** (`binance_cex`):

```bash
cd binance-sim
cargo run
# Health: curl -s http://127.0.0.1:8083/health
```

**Solo riel Solana** (`solana_wallet`) — consulta de saldo vía RPC:

```bash
# Opción A: validador local (default en oracle/.env)
solana-test-validator

# Opción B: devnet — cambiar SOLANA_RPC_URL en oracle/.env
# SOLANA_RPC_URL=https://api.devnet.solana.com
```

### Terminal 3 — Oracle

```bash
cd oracle
cargo run
# Health: curl -s http://127.0.0.1:8081/health
```

El Gateway verifica el health del Oracle al arrancar si `GATEWAY_ORACLE_HEALTH_CHECK=true`.

### Terminal 4 — API Gateway

```bash
cd crates/api-gateway
cargo run
# Health: curl -s http://127.0.0.1:8080/health
```

Liquidación en dev usa **adaptadores stub in-memory** (`SettlementEngine::with_stub_adapters()`). No hace falta levantar settlement real para checkout MVP.

### Terminal 5 — Frontend

```bash
cd frontend
pnpm install
pnpm dev
# UI: http://127.0.0.1:5173
```

### Terminal 6 — E2E Playwright (opcional)

Con el stack anterior en marcha:

```bash
cd tests/e2e
pnpm install
pnpm exec playwright install chromium
pnpm test
```

Ver [tests/e2e/README.md](../tests/e2e/README.md).

---

## 4. Tabla canónica de variables

### 4.1 Puertos y URLs

| Servicio | Puerto | Health | Variables host/puerto |
|----------|--------|--------|------------------------|
| API Gateway | 8080 | `GET /health` | `GATEWAY_HOST`, `GATEWAY_PORT` |
| Oracle | 8081 | `GET /health` | `ORACLE_HOST`, `ORACLE_PORT` |
| Antifraude | 8082 | `GET /health` | `ANTIFRAUD_HOST`, `ANTIFRAUD_PORT` |
| Binance sim | 8083 | `GET /health` | `BINANCE_SIM_HOST`, `BINANCE_SIM_PORT` |
| Frontend (Vite) | 5173 | — | `E2E_FRONTEND_PORT` (solo E2E) |

### 4.2 Enlaces entre servicios

| Variable | Valor local típico | Consumidor |
|----------|-------------------|------------|
| `ORACLE_BASE_URL` | `http://127.0.0.1:8081` | Gateway |
| `ANTIFRAUD_BASE_URL` | `http://127.0.0.1:8082` | Oracle |
| `BINANCE_CEX_BASE_URL` | `http://127.0.0.1:8083` | Oracle |
| `VITE_API_BASE_URL` | `http://127.0.0.1:8080` | Frontend (navegador) |
| `SOLANA_RPC_URL` | `http://127.0.0.1:8899` o devnet | Oracle |

### 4.3 Secretos que deben coincidir

| Grupo | Variables |
|-------|-----------|
| Gateway ↔ Oracle | `ORACLE_API_KEY` (Gateway) = `ORACLE_API_KEY` (Oracle) |
| Comercio demo | `GATEWAY_TEST_API_KEY` = `VITE_GATEWAY_API_KEY` |
| Binance | `BINANCE_CEX_API_KEY` en Oracle y binance-sim |
| Antifraude | `ANTIFRAUD_API_KEY` en Oracle y antifraud |

### 4.4 Archivos `.env` por servicio

| Ruta | Plantilla |
|------|-----------|
| `binance-sim/.env` | [`.env.example`](../binance-sim/.env.example) |
| `antifraud/.env` | [`.env.example`](../antifraud/.env.example) |
| `oracle/.env` | [`.env.example`](../oracle/.env.example) |
| `crates/api-gateway/.env` | [`.env.example`](../crates/api-gateway/.env.example) |
| `frontend/.env.local` | [`.env.example`](../frontend/.env.example) |
| `programs/payment-settlement/.env` | Solo Anchor/Solana on-chain (Fase 3) |

---

## 5. Matriz de stack por riel

| Riel | Servicios mínimos | Notas |
|------|-------------------|-------|
| `traditional_bank` | PG + antifraud + oracle + gateway + frontend | Saldo desde `TRADITIONAL_BANK_BALANCE` |
| `binance_cex` | + binance-sim | Saldo vía `GET /internal/v1/spot/balance` |
| `solana_wallet` | + RPC Solana accesible | Sin RPC → Oracle `503 RAIL_UNAVAILABLE` |

Tarjeta de prueba (Luhn válido): `4111111111111111`, expiry `12/2030`, CVV `123`.

---

## 6. Verificación rápida

### Script

```bash
./scripts/check-env.sh --live
```

### Manual

```bash
# 1. Healthchecks
curl -sf http://127.0.0.1:8082/health && echo " antifraude OK"
curl -sf http://127.0.0.1:8083/health && echo " binance-sim OK"    # si aplica
curl -sf http://127.0.0.1:8081/health && echo " oracle OK"
curl -sf http://127.0.0.1:8080/health && echo " gateway OK"

# 2. Checkout curl (banco)
export GATEWAY=http://127.0.0.1:8080
export API_KEY=sk_test_change_me_32chars_min   # tu GATEWAY_TEST_API_KEY

curl -s -X POST "$GATEWAY/api/v1/checkout" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: runbook-$(date +%s)" \
  -H "Content-Type: application/json" \
  -d @scripts/fixtures/checkout-bank.json | jq
```

### UI

1. Abrir http://127.0.0.1:5173  
2. **Usar datos de prueba** → **Validar tarjeta**  
3. Elegir riel → **Confirmar pago**  
4. Ver comprobante (`ACH-*`, `CEX-*`, `SOL-*`)

---

## 7. Troubleshooting

| Síntoma | Causa probable | Acción |
|---------|----------------|--------|
| Gateway falla al arrancar: Oracle health | Oracle caído o `ORACLE_API_KEY` distinta | Levantar Oracle; `./scripts/check-env.sh` |
| `ORACLE_API_KEY es obligatoria` desde raíz | `.env` no cargado (CWD incorrecto) | `cd crates/api-gateway && cargo run` |
| Oracle: error PostgreSQL | BD inexistente o URL incorrecta | `./oracle/scripts/setup-db.sh oracle` |
| Checkout 401 | API key frontend ≠ Gateway | Sincronizar `VITE_GATEWAY_API_KEY` |
| Checkout 503 antifraude | antifraud no corre | Terminal 2 — `cd antifraud && cargo run` |
| Riel Binance 503 | binance-sim caído o API key distinta | Levantar binance-sim; revisar `BINANCE_CEX_API_KEY` |
| Riel Solana 503 | RPC inaccesible | `solana-test-validator` o cambiar a devnet |
| Playwright skip stack real | Gateway no responde | Completar §3 antes de `pnpm test` |
| Puerto en uso | Servicio duplicado | `ss -tlnp \| grep 808` y detener proceso |

### Logs útiles

Cada servicio respeta `RUST_LOG` en su `.env`. Ejemplo:

```
RUST_LOG=api_gateway=debug,oracle_authorization=debug,tower_http=info
```

---

## 8. Comandos de referencia

```bash
# Tests Rust (requiere PostgreSQL para integración Oracle)
export ORACLE_DATABASE_URL=postgres://postgres:postgres@localhost:5432/oracle_test
cargo test --workspace -- --test-threads=1

# Frontend
cd frontend && pnpm test:run && pnpm lint && pnpm build

# Cross-service
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# CI local — ver Doc/CI.md
```

---

## 9. Referencias

| Documento | Contenido |
|-----------|-----------|
| [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening) | Pasos Fase 6 |
| [CI.md](./CI.md) | Pipeline GitHub Actions |
| [Checklist-QA-Fase-6.md](./Checklist-QA-Fase-6.md) | QA UC-01–UC-11 |
| [Deuda-Tecnica.md §5](./Deuda-Tecnica.md#5-configuración-y-entorno) | Variables compartidas |
| [tests/e2e/README.md](../tests/e2e/README.md) | Playwright E2E |
| `oracle/README.md` | Detalle Oracle y tests |
| `crates/api-gateway/README.md` | curl checkout |

---

*Última actualización: 2026-07-26 — Fase 6.7*
