# Staging — Docker Compose

Stack completo para Fase 7: PostgreSQL, antifraude, binance-sim, Oracle (red interna), Gateway y Caddy (TLS + frontend).

## Inicio rápido

```bash
cp .env.example .env   # o: ../../scripts/generate-staging-secrets.sh
# Editar secretos y dominio
../../scripts/check-staging-env.sh --preflight
../../scripts/deploy-staging.sh
```

Documentación: [Doc/Runbook-Staging.md](../../Doc/Runbook-Staging.md), [Doc/Provision-VPS-Fase-7.md](../../Doc/Provision-VPS-Fase-7.md).

## Scripts relacionados

| Script | Uso |
|--------|-----|
| `deploy/staging/provision-vps.sh` | Bootstrap Docker + UFW en VPS (como root) |
| `scripts/generate-staging-secrets.sh` | Genera `.env` con secretos aleatorios |
| `scripts/check-staging-env.sh` | Valida `.env` pre-deploy |
| `scripts/deploy-staging.sh` | Build + up + smoke banco |
| `scripts/smoke-staging.sh` | Smoke HTTP post-deploy |
| `scripts/verify-staging-isolation.sh` | Puertos internos cerrados |
| `scripts/setup-solana-devnet-staging.sh` | Verifica wallet SPL devnet |

## Arquitectura

```
Internet → Caddy (:443)
              ├─ /          → frontend estático (Vite build)
              ├─ /api/*     → gateway:8080 (interno)
              └─ /health    → gateway:8080

Red backend (sin puertos host):
  postgres, antifraud, binance-sim, oracle, gateway
```

## Perfiles

| Perfil | Comando | Requisito |
|--------|---------|-----------|
| **Native local** | `./scripts/staging-local.sh up` | Rust + PostgreSQL local |
| **Docker local** | `./scripts/staging-local.sh up --docker` | Docker + Compose |
| **TLS (VPS)** | `./scripts/deploy-staging.sh` | Dominio + DNS |

## Smoke tests

```bash
export STAGING_BASE_URL=https://staging.ejemplo.com
export GATEWAY_TEST_API_KEY=sk_test_...
../../scripts/smoke-staging.sh
```

## CI

El workflow `.github/workflows/staging-build.yml` valida que las imágenes compilen en cada push a `main`.
