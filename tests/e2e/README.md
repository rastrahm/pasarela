# E2E — Playwright (Fase 6.1)

Tests de extremo a extremo del checkout React contra el **stack real** (Gateway + Oracle).

## Requisitos

- Node.js 18+
- pnpm 9+
- Dependencias del frontend instaladas (`cd frontend && pnpm install`)

## Instalación

```bash
cd tests/e2e
pnpm install
pnpm exec playwright install chromium
```

## Stack para tests contra backend real

Orden recomendado (sin contenedores):

```bash
# Terminal 1 — simuladores (según riel)
cargo run -p binance-sim-service   # :8083 si usás Binance
cargo run -p antifraud-service     # :8082

# Terminal 2 — Oracle
cd oracle && cargo run             # :8081

# Terminal 3 — Gateway
cargo run -p api-gateway           # :8080

# Terminal 4 — E2E (levanta frontend automáticamente)
cd tests/e2e && pnpm test
```

Variables alineadas con `frontend/.env.example` y `crates/api-gateway/.env.example`:

| Variable | Default |
|----------|---------|
| `VITE_API_BASE_URL` | `http://127.0.0.1:8080` |
| `VITE_GATEWAY_API_KEY` | `sk_test_change_me_32chars_min` |

Copiá `frontend/.env.example` → `frontend/.env.local` si usás otros valores.

## Ejecución local

```bash
cd tests/e2e
pnpm test              # headless
pnpm test:headed       # navegador visible
pnpm test:ui           # UI mode
pnpm report            # ver último reporte HTML
```

## Ejecución en CI

El job `e2e` de [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) ejecuta esta suite en cada push/PR (Node 20 + Chromium). Ver [Doc/CI.md](../../Doc/CI.md).

## Comportamiento

| Suite | Backend requerido |
|-------|-------------------|
| `Checkout E2E — stack real` | Sí — se omite si `/health` del Gateway falla |
| `Checkout E2E — validación UI` | No — solo frontend |

## Estructura

```
tests/e2e/
├── playwright.config.ts
├── specs/checkout.spec.ts
├── helpers/checkout.ts
└── README.md
```

## Referencias

- [Doc/Plan-de-Implementacion.md §10](../../Doc/Plan-de-Implementacion.md)
- [Doc/Deuda-Tecnica.md](../../Doc/Deuda-Tecnica.md)
- [frontend/README.md](../../frontend/README.md)
