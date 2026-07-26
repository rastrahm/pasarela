# CI — Pasarela Multi-Rail

> Fase 6.6 · Workflows GitHub Actions

## Workflows

| Archivo | Trigger | Jobs |
|---------|---------|------|
| [`ci.yml`](../.github/workflows/ci.yml) | Push/PR `main`, `master`, `liquidacion` | Rust, Frontend, Playwright |
| [`programs-anchor-test.yml`](../.github/workflows/programs-anchor-test.yml) | Cambios en `programs/payment-settlement/` | Anchor + Solana local validator |

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
- Tests de stack real se **omitien** si Gateway no está (SKIP)
- Suite UI validación corre siempre

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

- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Revision-Rendimiento-Fase-6.md](./Revision-Rendimiento-Fase-6.md)
- [tests/e2e/README.md](../tests/e2e/README.md)
