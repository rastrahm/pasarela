# E2E — Playwright (Fase 6.1)

Tests de extremo a extremo del checkout React contra el **stack real** (Gateway + Oracle + simuladores).

> Guía general: [Doc/Pruebas.md](../../Doc/Pruebas.md) · Stack: [Doc/Runbook-Desarrollo.md](../../Doc/Runbook-Desarrollo.md)

## Requisitos

- Node.js 18+ (CI usa 20)
- pnpm 9+
- `@playwright/test@1.59.1` (compatible Node 18; ver DT-P1-06)
- Stack local levantado para tests **stack real** (ver abajo)
- `.env` sincronizados: `./scripts/check-env.sh --live`

## Instalación

```bash
cd tests/e2e
pnpm install
pnpm exec playwright install chromium
```

## Stack para tests contra backend real

**Importante:** ejecutá cada servicio Rust desde su carpeta (`.env` vía `dotenvy`).

Orden — detalle en [Runbook-Desarrollo.md](../../Doc/Runbook-Desarrollo.md):

```bash
# Terminal 1 — Solana (solo riel solana_wallet)
solana-test-validator --quiet --reset

# Terminal 2 — simuladores
cd antifraud && cargo run                    # :8082
cd binance-sim && cargo run                  # :8083

# Terminal 3 — Oracle (desde oracle/)
cd oracle && cargo run                       # :8081

# Terminal 4 — Gateway (desde crates/api-gateway/)
cd crates/api-gateway && cargo run           # :8080

# Terminal 5 — E2E (Playwright levanta Vite en :5173)
cd tests/e2e && pnpm test
```

### Variables

| Variable | Default | Archivo |
|----------|---------|---------|
| `VITE_API_BASE_URL` | `http://127.0.0.1:8080` | `frontend/.env.local` |
| `VITE_GATEWAY_API_KEY` | `sk_test_change_me_32chars_min` | debe = `GATEWAY_TEST_API_KEY` |

Copiá `frontend/.env.example` → `frontend/.env.local` y `crates/api-gateway/.env.example` → `crates/api-gateway/.env`.

### CORS (obligatorio para E2E navegador)

El frontend en `:5173` llama al Gateway en `:8080`. El Gateway incluye `CorsLayer` para orígenes locales de desarrollo. Si falta, los tests de stack real fallan con *“No se pudo contactar al API Gateway”* en la UI (aunque `curl` al API funcione).

### Solana local

Para `checkout exitoso — riel solana_wallet`:

1. `solana-test-validator` en `:8899`
2. Oracle con `SOLANA_RPC_URL=http://127.0.0.1:8899`
3. Pubkey **válido** en `SOLANA_WALLET_PUBKEY` (no el placeholder `DemoWallet…`)
4. Mint SPL local con saldo — ver [Runbook §8](../../Doc/Runbook-Desarrollo.md#8-anexo--solana-local-para-e2e)

## Suites y tests

| Suite | Tests | Backend |
|-------|-------|---------|
| `Checkout E2E — stack real` | 4 | Gateway + Oracle + simuladores |
| `Checkout E2E — validación UI` | 2 | Solo Vite (sin Gateway) |

### Stack real (4 tests)

1. Muestra página checkout
2. `traditional_bank` → comprobante `ACH-*`
3. `binance_cex` → comprobante `CEX-*`
4. `solana_wallet` → comprobante `SOL-MEM-*`

Verificación local **2026-07-26:** 6/6 pasando con stack completo.

Los tests de stack real se **omitien** (`test.skip`) si `GET /health` del Gateway falla en `beforeAll`.

## Ejecución local

```bash
cd tests/e2e
pnpm test              # headless — 6 tests
pnpm test:headed       # navegador visible
pnpm test:ui           # UI mode
pnpm report            # ver último reporte HTML
```

Filtrar solo stack real:

```bash
pnpm exec playwright test --grep "stack real"
```

## Ejecución en CI

Job `e2e` en [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — Node 20 + Chromium.

- Levanta Vite automáticamente (`playwright.config.ts`)
- Corre **validación UI** (2 tests) siempre
- **Stack real:** skip si no hay Gateway en el runner (no se levanta stack Rust en CI hoy)

Ver [Doc/CI.md](../../Doc/CI.md).

## Estructura

```
tests/e2e/
├── playwright.config.ts   # webServer Vite + env VITE_*
├── specs/checkout.spec.ts # 6 tests — 3 rieles + UI
├── helpers/checkout.ts    # isGatewayHealthy, checkoutWithRail, …
└── README.md
```

## Aserciones

Tras checkout exitoso, las aserciones de riel y proof se limitan al panel **Comprobante** (evita ambigüedad con el selector de riel y el log).

## Referencias

- [Doc/Pruebas.md](../../Doc/Pruebas.md)
- [Doc/Runbook-Desarrollo.md](../../Doc/Runbook-Desarrollo.md)
- [scripts/fixtures/README.md](../../scripts/fixtures/README.md)
- [Doc/Checklist-QA-Fase-6.md](../../Doc/Checklist-QA-Fase-6.md)
