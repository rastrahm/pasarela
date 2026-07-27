# E2E — Playwright (Phase 6.1)

End-to-end tests for the React checkout against the **real stack** (Gateway + Oracle + simulators).

> General guide: [Doc/Pruebas-en.md](../../Doc/Pruebas-en.md) · Stack: [Doc/Runbook-Desarrollo-en.md](../../Doc/Runbook-Desarrollo-en.md)

## Requirements

- Node.js 18+ (CI uses 20)
- pnpm 9+
- `@playwright/test@1.59.1` (Node 18 compatible; see DT-P1-06)
- Local stack running for **real stack** tests (see below)
- Synchronized `.env` files: `./scripts/check-env.sh --live`

## Installation

```bash
cd tests/e2e
pnpm install
pnpm exec playwright install chromium
```

## Stack for real backend tests

**Important:** run each Rust service from its folder (`.env` via `dotenvy`).

Order — details in [Runbook-Desarrollo-en.md](../../Doc/Runbook-Desarrollo-en.md):

```bash
# Terminal 1 — Solana (solana_wallet rail only)
solana-test-validator --quiet --reset

# Terminal 2 — simulators
cd antifraud && cargo run                    # :8082
cd binance-sim && cargo run                  # :8083

# Terminal 3 — Oracle (from oracle/)
cd oracle && cargo run                       # :8081

# Terminal 4 — Gateway (from crates/api-gateway/)
cd crates/api-gateway && cargo run           # :8080

# Terminal 5 — E2E (Playwright starts Vite on :5173)
cd tests/e2e && pnpm test
```

### Variables

| Variable | Default | File |
|----------|---------|---------|
| `VITE_API_BASE_URL` | `http://127.0.0.1:8080` | `frontend/.env.local` |
| `VITE_GATEWAY_API_KEY` | `sk_test_change_me_32chars_min` | must = `GATEWAY_TEST_API_KEY` |

Copy `frontend/.env.example` → `frontend/.env.local` and `crates/api-gateway/.env.example` → `crates/api-gateway/.env`.

### CORS (required for browser E2E)

The frontend on `:5173` calls the Gateway on `:8080`. The Gateway includes `CorsLayer` for local development origins. If missing, real stack tests fail with *"Could not contact the API Gateway"* in the UI (even if `curl` to the API works).

### Local Solana

For `successful checkout — solana_wallet rail`:

1. `solana-test-validator` on `:8899`
2. Oracle with `SOLANA_RPC_URL=http://127.0.0.1:8899`
3. Valid pubkey in `SOLANA_WALLET_PUBKEY` (not the `DemoWallet…` placeholder)
4. Local SPL mint with balance — see [Runbook §8](../../Doc/Runbook-Desarrollo-en.md#8-anexo--solana-local-para-e2e)

## Suites and tests

| Suite | Tests | Backend |
|-------|-------|---------|
| `Checkout E2E — stack real` | 4 | Gateway + Oracle + simulators |
| `Checkout E2E — validación UI` | 2 | Vite only (no Gateway) |

### Real stack (4 tests)

1. Displays checkout page
2. `traditional_bank` → `ACH-*` receipt
3. `binance_cex` → `CEX-*` receipt
4. `solana_wallet` → `SOL-MEM-*` receipt

Local verification **2026-07-26:** 6/6 passing with full stack.

Real stack tests are **skipped** (`test.skip`) if Gateway `GET /health` fails in `beforeAll`.

## Local execution

```bash
cd tests/e2e
pnpm test              # headless — 6 tests
pnpm test:headed       # visible browser
pnpm test:ui           # UI mode
pnpm report            # view last HTML report
```

Filter real stack only:

```bash
pnpm exec playwright test --grep "stack real"
```

## CI execution

Job `e2e` in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — Node 20 + Chromium.

- Starts Vite automatically (`playwright.config.ts`)
- Runs **UI validation** (2 tests) always
- **Real stack:** skip if no Gateway on runner (Rust stack not started in CI today)

See [Doc/CI-en.md](../../Doc/CI-en.md).

## Structure

```
tests/e2e/
├── playwright.config.ts   # Vite webServer + VITE_* env
├── specs/checkout.spec.ts # 6 tests — 3 rails + UI
├── helpers/checkout.ts    # isGatewayHealthy, checkoutWithRail, …
└── README-en.md
```

## Assertions

After successful checkout, rail and proof assertions are limited to the **Receipt** panel (avoids ambiguity with the rail selector and log).

## References

- [Doc/Pruebas-en.md](../../Doc/Pruebas-en.md)
- [Doc/Runbook-Desarrollo-en.md](../../Doc/Runbook-Desarrollo-en.md)
- [scripts/fixtures/README-en.md](../../scripts/fixtures/README-en.md)
- [Doc/Checklist-QA-Fase-6-en.md](../../Doc/Checklist-QA-Fase-6-en.md)
