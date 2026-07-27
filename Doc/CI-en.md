# CI — Pasarela Multi-Rail

> Phase 6.6 · GitHub Actions Workflows

## Workflows

| File | Trigger | Jobs |
|------|---------|------|
| [`ci.yml`](../.github/workflows/ci.yml) | Push/PR `main`, `master`, `liquidacion` | Rust, Frontend, Playwright |
| [`programs-anchor-test.yml`](../.github/workflows/programs-anchor-test.yml) | Changes in `programs/payment-settlement/` | Anchor + Solana local validator |
| [`staging-build.yml`](../.github/workflows/staging-build.yml) | Push/PR (staging paths + crates + frontend) | Build staging Docker images |
| [`deploy-staging.yml`](../.github/workflows/deploy-staging.yml) | Manual (`workflow_dispatch`) | SSH deploy to staging VPS |

## Job `CI / Rust`

```bash
# Local equivalent (requires PostgreSQL)
export ORACLE_DATABASE_URL=postgres://postgres:postgres@localhost:5432/oracle_test
cargo test --workspace -- --test-threads=1
```

- Postgres 16 service in CI (`oracle_test`)
- Includes: `domain`, `api-gateway`, `oracle`, `antifraud`, `binance-sim`, etc.
- `cross_service_integration` and Oracle tests use the same database

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

- Starts Vite automatically (`playwright.config.ts`)
- **Always runs:** 2 UI validation tests (no backend)
- **Real stack (4 tests):** **skipped** if Gateway is not on the runner
- The 3 full-stack E2E rails are verified **locally** — see [Pruebas-en.md §3](./Pruebas-en.md#3-regression-with-local-stack)

> The `e2e` job does not start Oracle/Gateway/simulators. For future full-stack E2E in CI, consider compose services or a dedicated job.

## Anchor (separate)

```bash
cd programs/payment-settlement
npm ci
npm run lint:docs
npm run test:ci   # or anchor test --skip-build
```

## Badges (optional in root README)

```markdown
![CI](https://github.com/<org>/pasarela/actions/workflows/ci.yml/badge.svg)
![Anchor](https://github.com/<org>/pasarela/actions/workflows/programs-anchor-test.yml/badge.svg)
```

## References

- [README.md](../README.md) — monorepo entry point
- [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening)
- [Revision-Rendimiento-Fase-6-en.md](./Revision-Rendimiento-Fase-6-en.md)
- [Pruebas-en.md](./Pruebas-en.md)
- [tests/e2e/README.md](../tests/e2e/README.md)
- [Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md)
- [Runbook-Staging-en.md](./Runbook-Staging-en.md)
- [Provision-VPS-Fase-7-en.md](./Provision-VPS-Fase-7-en.md)

## `Deploy Staging` Workflow

Manual trigger. Secrets in GitHub:

| Secret | Description |
|--------|-------------|
| `STAGING_SSH_HOST` | VPS IP or hostname |
| `STAGING_SSH_USER` | SSH user (e.g. `deploy`) |
| `STAGING_SSH_KEY` | Private PEM key |

The VPS must have the repo cloned and `deploy/staging/.env` configured before the first remote deploy.
