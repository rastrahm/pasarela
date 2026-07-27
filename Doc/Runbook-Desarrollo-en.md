# Development Runbook — Pasarela Multi-Rail

> Phase 6.7 · Local stack without containers  
> Resolves [DT-P0-03](./Deuda-Tecnica-en.md#dt-p0-03--dispersed-local-stack-runbook) and canonical table from [DT-P0-04](./Deuda-Tecnica-en.md#dt-p0-04--environment-variables-out-of-sync-between-services).

## 1. Prerequisites

| Tool | Minimum version | Use |
|------|-----------------|-----|
| Rust | 1.75+ | Gateway, Oracle, simulators |
| PostgreSQL | 14+ (16 in CI) | Oracle (required) |
| Node.js | 18+ (20 in CI) | Frontend, Playwright |
| pnpm | 9+ | Frontend, E2E |
| `curl`, `jq` | any | Manual healthchecks |

Optional by rail:

| Tool | Rail |
|------|------|
| Solana CLI + local validator | `solana_wallet` (RPC balance query in Oracle) |
| Anchor | Only `programs/payment-settlement/` (outside checkout MVP stack) |

---

## 2. Initial setup (once)

### 2.1 Copy environment files

From monorepo root:

```bash
cp binance-sim/.env.example   binance-sim/.env
cp antifraud/.env.example     antifraud/.env
cp oracle/.env.example        oracle/.env
cp crates/api-gateway/.env.example crates/api-gateway/.env
cp frontend/.env.example      frontend/.env.local
```

> **Important:** `dotenvy` loads `.env` from the **current working directory**. Run each Rust service from its folder (see §3) or export variables before `cargo run -p …` from root.

### 2.2 Sync shared secrets

Generate a 32-byte hex key and use it in all indicated pairs:

```bash
openssl rand -hex 32   # example: a1b2c3...
```

| Variable pair | Files |
|---------------|-------|
| `ORACLE_API_KEY` | `crates/api-gateway/.env` ↔ `oracle/.env` |
| `GATEWAY_TEST_API_KEY` ↔ `VITE_GATEWAY_API_KEY` | `crates/api-gateway/.env` ↔ `frontend/.env.local` |
| `BINANCE_CEX_API_KEY` | `oracle/.env` ↔ `binance-sim/.env` |
| `ANTIFRAUD_API_KEY` | `oracle/.env` ↔ `antifraud/.env` |

Automatic validation:

```bash
chmod +x scripts/check-env.sh
./scripts/check-env.sh          # .env files only
./scripts/check-env.sh --live   # + HTTP healthchecks
```

### 2.3 Oracle database

```bash
cd oracle
chmod +x scripts/setup-db.sh
./scripts/setup-db.sh oracle
# Adjust ORACLE_DATABASE_URL in oracle/.env if user/password differ
```

SQLx migrations apply on Oracle startup.

### 2.4 Gateway without PostgreSQL (optional)

Gateway can use **in-memory** storage if you comment out or remove `DATABASE_URL` in `crates/api-gateway/.env`. Useful for quick tests; persistence between restarts is not preserved.

| Service | Default DB | Required |
|---------|------------|----------|
| Oracle | `postgres://postgres:postgres@localhost:5432/oracle` | Yes |
| Gateway | `postgres://pasarela:pasarela@127.0.0.1:5432/pasarela_gateway` | No (in-memory fallback) |

---

## 3. Startup order

Service dependencies:

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

Ensure PostgreSQL listens on `localhost:5432` and database `oracle` exists.

### Terminal 2 — Simulators (by rail)

**Always recommended** (Oracle queries antifraud on every authorization):

```bash
cd antifraud
cargo run
# Health: curl -s http://127.0.0.1:8082/health
```

**Binance rail only** (`binance_cex`):

```bash
cd binance-sim
cargo run
# Health: curl -s http://127.0.0.1:8083/health
```

**Solana rail only** (`solana_wallet`) — balance query via RPC:

```bash
# Option A: local validator (default in oracle/.env)
solana-test-validator

# Option B: devnet — change SOLANA_RPC_URL in oracle/.env
# SOLANA_RPC_URL=https://api.devnet.solana.com
```

### Terminal 3 — Oracle

```bash
cd oracle
cargo run
# Health: curl -s http://127.0.0.1:8081/health
```

Gateway checks Oracle health on startup if `GATEWAY_ORACLE_HEALTH_CHECK=true`.

### Terminal 4 — API Gateway

```bash
cd crates/api-gateway
cargo run
# Health: curl -s http://127.0.0.1:8080/health
```

Settlement in dev uses **in-memory stub adapters** (`SettlementEngine::with_stub_adapters()`). No need to start real settlement for checkout MVP.

### Terminal 5 — Frontend

```bash
cd frontend
pnpm install
pnpm dev
# UI: http://127.0.0.1:5173
```

### Terminal 6 — Playwright E2E (optional)

With the stack above running:

```bash
cd tests/e2e
pnpm install
pnpm exec playwright install chromium
pnpm test
```

See [tests/e2e/README.md](../tests/e2e/README.md).

---

## 4. Canonical variable table

### 4.1 Ports and URLs

| Service | Port | Health | Host/port variables |
|---------|------|--------|----------------------|
| API Gateway | 8080 | `GET /health` | `GATEWAY_HOST`, `GATEWAY_PORT` |
| Oracle | 8081 | `GET /health` | `ORACLE_HOST`, `ORACLE_PORT` |
| Antifraud | 8082 | `GET /health` | `ANTIFRAUD_HOST`, `ANTIFRAUD_PORT` |
| Binance sim | 8083 | `GET /health` | `BINANCE_SIM_HOST`, `BINANCE_SIM_PORT` |
| Frontend (Vite) | 5173 | — | `E2E_FRONTEND_PORT` (E2E only) |

### 4.2 Inter-service links

| Variable | Typical local value | Consumer |
|----------|---------------------|----------|
| `ORACLE_BASE_URL` | `http://127.0.0.1:8081` | Gateway |
| `ANTIFRAUD_BASE_URL` | `http://127.0.0.1:8082` | Oracle |
| `BINANCE_CEX_BASE_URL` | `http://127.0.0.1:8083` | Oracle |
| `VITE_API_BASE_URL` | `http://127.0.0.1:8080` | Frontend (browser) |
| `SOLANA_RPC_URL` | `http://127.0.0.1:8899` or devnet | Oracle |

### 4.3 Secrets that must match

| Group | Variables |
|-------|-----------|
| Gateway ↔ Oracle | `ORACLE_API_KEY` (Gateway) = `ORACLE_API_KEY` (Oracle) |
| Demo merchant | `GATEWAY_TEST_API_KEY` = `VITE_GATEWAY_API_KEY` |
| Binance | `BINANCE_CEX_API_KEY` in Oracle and binance-sim |
| Antifraud | `ANTIFRAUD_API_KEY` in Oracle and antifraud |

### 4.4 `.env` files per service

| Path | Template |
|------|----------|
| `binance-sim/.env` | [`.env.example`](../binance-sim/.env.example) |
| `antifraud/.env` | [`.env.example`](../antifraud/.env.example) |
| `oracle/.env` | [`.env.example`](../oracle/.env.example) |
| `crates/api-gateway/.env` | [`.env.example`](../crates/api-gateway/.env.example) |
| `frontend/.env.local` | [`.env.example`](../frontend/.env.example) |
| `programs/payment-settlement/.env` | Anchor/Solana on-chain only (Phase 3) |

---

## 5. Stack matrix by rail

| Rail | Minimum services | Notes |
|------|------------------|-------|
| `traditional_bank` | PG + antifraud + oracle + gateway + frontend | Balance from `TRADITIONAL_BANK_BALANCE` |
| `binance_cex` | + binance-sim | Balance via `GET /internal/v1/spot/balance` |
| `solana_wallet` | + accessible Solana RPC + valid pubkey/mint | See [§8 Solana Appendix](#8-appendix--local-solana-for-e2e) |

Test card (valid Luhn): `4111111111111111`, expiry `12/2030`, CVV `123`.

---

## 6. Quick verification

### Script

```bash
./scripts/check-env.sh --live
```

### Manual

```bash
# 1. Healthchecks
curl -sf http://127.0.0.1:8082/health && echo " antifraud OK"
curl -sf http://127.0.0.1:8083/health && echo " binance-sim OK"    # if applicable
curl -sf http://127.0.0.1:8081/health && echo " oracle OK"
curl -sf http://127.0.0.1:8080/health && echo " gateway OK"

# 2. Checkout curl (bank)
export GATEWAY=http://127.0.0.1:8080
export API_KEY=sk_test_change_me_32chars_min   # your GATEWAY_TEST_API_KEY

# 3. Checkout curl — 3 rails
for f in checkout-bank.json checkout-binance.json checkout-solana.json; do
  echo "=== $f ==="
  curl -s -X POST "$GATEWAY/api/v1/checkout" \
    -H "Authorization: Bearer $API_KEY" \
    -H "Idempotency-Key: runbook-$(date +%s)-$RANDOM" \
    -H "Content-Type: application/json" \
    -d @scripts/fixtures/$f | jq
done

# 4. Playwright E2E (6 tests — requires stack + CORS)
cd tests/e2e && pnpm test
```

### UI

1. Open http://127.0.0.1:5173  
2. **Use test data** → **Validate card**  
3. Choose rail → **Confirm payment**  
4. View receipt (`ACH-*`, `CEX-*`, `SOL-*`)

---

## 7. Troubleshooting

| Symptom | Likely cause | Action |
|---------|--------------|--------|
| Gateway fails on startup: Oracle health | Oracle down or different `ORACLE_API_KEY` | Start Oracle; `./scripts/check-env.sh` |
| `ORACLE_API_KEY is required` from root | `.env` not loaded (wrong CWD) | `cd crates/api-gateway && cargo run` |
| Oracle: PostgreSQL error | Missing DB or wrong URL | `./oracle/scripts/setup-db.sh oracle` |
| Checkout 401 | Frontend API key ≠ Gateway | Sync `VITE_GATEWAY_API_KEY` |
| Checkout 503 antifraud | antifraud not running | Terminal 2 — `cd antifraud && cargo run` |
| Binance rail 503 | binance-sim down or different API key | Start binance-sim; check `BINANCE_CEX_API_KEY` |
| Solana rail 503 | RPC down or invalid pubkey | §8 Solana Appendix |
| Solana rail 402 | No local SPL balance | Mint tokens §8 |
| Playwright: *No connection to Gateway* in UI | CORS or Gateway down | Dev CORS on Gateway; `:8080/health` |
| Playwright skip real stack | Gateway not responding | Complete §3; `./scripts/check-env.sh --live` |
| Port in use | Duplicate service | `ss -tlnp \| grep 808` and stop process |

### Useful logs

Each service respects `RUST_LOG` in its `.env`. Example:

```
RUST_LOG=api_gateway=debug,oracle_authorization=debug,tower_http=info
```

---

## 8. Reference commands

```bash
# Rust tests (requires PostgreSQL for Oracle integration)
export ORACLE_DATABASE_URL=postgres://postgres:postgres@localhost:5432/oracle_test
cargo test --workspace -- --test-threads=1

# Frontend
cd frontend && pnpm test:run && pnpm lint && pnpm build

# Cross-service
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# Local CI — see Doc/CI-en.md
```

---

## 8. Appendix — Local Solana for E2E

The placeholder `DemoWallet1111111111111111111111111111111` is **not valid** for JSON-RPC. For `solana_wallet` checkout on localnet:

```bash
# 1. Validator
solana-test-validator --quiet --reset

# 2. Test wallet
solana-keygen new --no-bip39-passphrase -o /tmp/pasarela-test-wallet.json --force
PUBKEY=$(solana-keygen pubkey /tmp/pasarela-test-wallet.json)

# 3. SPL mint + balance
solana config set --url http://127.0.0.1:8899
solana airdrop 2 "$PUBKEY"
MINT=$(spl-token create-token --decimals 6 2>&1 | awk '/Creating token/{print $3}')
spl-token create-account "$MINT" --owner /tmp/pasarela-test-wallet.json
ATA=$(spl-token accounts --owner "$PUBKEY" | awk 'NR==3{print $1}')
spl-token mint "$MINT" 10000 "$ATA"
```

Update `oracle/.env`:

```
SOLANA_RPC_URL=http://127.0.0.1:8899
SOLANA_WALLET_PUBKEY=<PUBKEY>
SOLANA_TOKEN_MINT=<MINT>
```

Restart Oracle. Verification: [Doc/Pruebas-en.md §3](./Pruebas-en.md#3-regression-with-local-stack).

---

## 9. References

| Document | Content |
|----------|---------|
| [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening) | Phase 6 steps |
| [CI-en.md](./CI-en.md) | GitHub Actions pipeline |
| [Checklist-QA-Fase-6-en.md](./Checklist-QA-Fase-6-en.md) | QA UC-01–UC-11 |
| [Deuda-Tecnica-en.md §5](./Deuda-Tecnica-en.md#5-configuration-and-environment) | Shared variables |
| [Pruebas-en.md](./Pruebas-en.md) | Unified testing guide (Phase 6) |
| [tests/e2e/README.md](../tests/e2e/README.md) | Playwright E2E |
| `oracle/README.md` | Oracle details and tests |
| `crates/api-gateway/README.md` | curl checkout |

---

*Last updated: 2026-07-26 — Phase 6.7*
