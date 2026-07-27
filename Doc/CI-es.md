# CI — Pasarela Multi-Rail

> Fase 6.6 · Workflows GitHub Actions

## Workflows

| Archivo | Trigger | Jobs |
|---------|---------|------|
| [`ci.yml`](../.github/workflows/ci.yml) | Push/PR `main`, `master`, `liquidacion` | Rust, Frontend, Playwright |
| [`programs-anchor-test.yml`](../.github/workflows/programs-anchor-test.yml) | Cambios en `programs/payment-settlement/` | Anchor + Solana local validator |
| [`staging-build.yml`](../.github/workflows/staging-build.yml) | Push/PR (paths staging + crates + frontend) | Build imágenes Docker staging |
| [`deploy-staging.yml`](../.github/workflows/deploy-staging.yml) | Manual (`workflow_dispatch`) | Deploy SSH al VPS staging |

## Job `CI / Rust`

```bash
# Equivalente local (requiere PostgreSQL)
export ORACLE_DATABASE_URL=postgres://postgres:postgres@localhost:5432/oracle_test
cargo test --workspace -- --test-threads=1
```

- Servicio Postgres 16 en CI (`oracle_test`)
- Incluye: `domain`, `api-gateway`, `oracle`, `antifraud`, `binance-sim`, etc.
- `cross_service_integration` y tests Oracle usan la misma BD

## Job `CI / Frontend`

```bash
cd frontend
pnpm install --frozen-lockfile
pnpm lint
pnpm test:run
pnpm build
```

## Job `CI / Playwright`

```bash
cd frontend && pnpm install --frozen-lockfile
cd tests/e2e
pnpm install --frozen-lockfile
pnpm exec playwright install chromium
pnpm test
```

- Levanta Vite automáticamente (`playwright.config.ts`)
- **Siempre corre:** 2 tests de validación UI (sin backend)
- **Stack real (4 tests):** se **omitien** si Gateway no está en el runner
- Los 3 rieles E2E con stack completo se verifican **localmente** — ver [Pruebas.md §3](./Pruebas.md#3-regresión-con-stack-local)

> El job `e2e` no levanta Oracle/Gateway/simuladores. Para E2E full-stack en CI futuro, considerar servicios compose o job dedicado.

## Anchor (separado)

```bash
cd programs/payment-settlement
npm ci
npm run lint:docs
npm run test:ci   # o anchor test --skip-build
```

## Badges (opcional en README raíz)

```markdown
![CI](https://github.com/<org>/pasarela/actions/workflows/ci.yml/badge.svg)
![Anchor](https://github.com/<org>/pasarela/actions/workflows/programs-anchor-test.yml/badge.svg)
```

## Referencias

- [README.md](../README.md) — entrada al monorepo
- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Revision-Rendimiento-Fase-6.md](./Revision-Rendimiento-Fase-6.md)
- [Pruebas.md](./Pruebas.md)
- [tests/e2e/README.md](../tests/e2e/README.md)
- [Runbook-Desarrollo.md](./Runbook-Desarrollo.md)
- [Runbook-Staging.md](./Runbook-Staging.md)
- [Provision-VPS-Fase-7.md](./Provision-VPS-Fase-7.md)

## Workflow `Deploy Staging`

Disparo manual. Secrets en GitHub:

| Secret | Descripción |
|--------|-------------|
| `STAGING_SSH_HOST` | IP o hostname del VPS |
| `STAGING_SSH_USER` | Usuario SSH (ej. `deploy`) |
| `STAGING_SSH_KEY` | Clave privada PEM |

El VPS debe tener el repo clonado y `deploy/staging/.env` configurado antes del primer deploy remoto.
