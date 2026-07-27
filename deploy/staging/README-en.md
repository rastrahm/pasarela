# Staging — Docker Compose

Full stack for Phase 7: PostgreSQL, antifraud, binance-sim, Oracle (internal network), Gateway, and Caddy (TLS + frontend).

## Quick start

```bash
cp .env.example .env   # or: ../../scripts/generate-staging-secrets.sh
# Edit secrets and domain
../../scripts/check-staging-env.sh --preflight
../../scripts/deploy-staging.sh
```

Documentation: [Doc/Runbook-Staging-en.md](../../Doc/Runbook-Staging-en.md), [Doc/Provision-VPS-Fase-7-en.md](../../Doc/Provision-VPS-Fase-7-en.md).

## Related scripts

| Script | Purpose |
|--------|---------|
| `deploy/staging/provision-vps.sh` | Bootstrap Docker + UFW on VPS (as root) |
| `scripts/generate-staging-secrets.sh` | Generate `.env` with random secrets |
| `scripts/check-staging-env.sh` | Validate `.env` before deploy |
| `scripts/deploy-staging.sh` | Build + up + bank smoke test |
| `scripts/smoke-staging.sh` | HTTP smoke test after deploy |
| `scripts/verify-staging-isolation.sh` | Internal ports closed from outside |
| `scripts/setup-solana-devnet-staging.sh` | Verify SPL devnet wallet |

## Architecture

```
Internet → Caddy (:443)
              ├─ /          → static frontend (Vite build)
              ├─ /api/*     → gateway:8080 (internal)
              └─ /health    → gateway:8080

Backend network (no host ports):
  postgres, antifraud, binance-sim, oracle, gateway
```

## Profiles

| Profile | Command | Requirement |
|---------|---------|-------------|
| **Native local** | `./scripts/staging-local.sh up` | Local Rust + PostgreSQL |
| **Docker local** | `./scripts/staging-local.sh up --docker` | Docker + Compose |
| **TLS (VPS)** | `./scripts/deploy-staging.sh` | Domain + DNS |

## Smoke tests

```bash
export STAGING_BASE_URL=https://staging.example.com
export GATEWAY_TEST_API_KEY=sk_test_...
../../scripts/smoke-staging.sh
```

## CI

The `.github/workflows/staging-build.yml` workflow validates that images build on every push to `main`.
