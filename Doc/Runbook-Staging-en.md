# Runbook — Staging (Phase 7)

> Pre-production environment: **1 VPS + Docker Compose + Caddy TLS**.  
> Oracle on internal network (no public port). Frontend and API same-origin via Caddy.

---

## 1. Prerequisites

| Requirement | Detail |
|-------------|--------|
| VPS | 2 vCPU, 4 GB RAM minimum; Ubuntu 22.04+ or Debian 12 |
| Docker | Engine 24+ and Compose v2 |
| Domain | DNS `A` / `AAAA` pointing to VPS (Let's Encrypt TLS) |
| Ports | 80 and 443 open to Caddy |
| Devnet | `https://api.devnet.solana.com` reachable from VPS |

---

## 2. First deploy

Complete step-by-step guide: **[Provision-VPS-Fase-7-en.md](./Provision-VPS-Fase-7-en.md)** (Block 2).

### 2.1 Clone and configure secrets

```bash
git clone <repo-url> pasarela && cd pasarela
./scripts/generate-staging-secrets.sh
# Edit deploy/staging/.env — domain, devnet, ACME_EMAIL
./scripts/check-staging-env.sh --preflight
openssl rand -hex 32   # only if regenerating secrets manually
```

Critical variables:

- `STAGING_DOMAIN` — public domain (e.g. `staging.pasarela.example.com`)
- `PUBLIC_BASE_URL` — `https://<STAGING_DOMAIN>` (frontend build)
- `POSTGRES_PASSWORD`, `ORACLE_API_KEY`, shared keys
- `SOLANA_WALLET_PUBKEY` + `SOLANA_TOKEN_MINT` — wallet with SPL on devnet

Anchor program on devnet: `programs/payment-settlement/deploy/devnet.json`.

### 2.2 Solana devnet (`solana_wallet` rail)

1. Create devnet wallet and fund SOL (`solana airdrop 2`).
2. Create or use test SPL mint; credit tokens to wallet.
3. Copy pubkey and mint to `deploy/staging/.env`.
4. Solana rail smoke: `./scripts/smoke-staging.sh --rail solana`.

### 2.3 Start stack (TLS)

```bash
./scripts/deploy-staging.sh
# manual equivalent:
# cd deploy/staging && docker compose --env-file .env --profile tls up -d --build
docker compose --env-file deploy/staging/.env -f deploy/staging/docker-compose.yml ps
```

Internal services (no host port): `oracle`, `antifraud`, `binance-sim`, `postgres`, `gateway`.  
Single entry point: **Caddy** (:80 / :443).

### 2.4 Local QA without domain (`local` profile)

**Without Docker** (recommended in dev — uses existing `cargo` stack):

```bash
./scripts/staging-local.sh init    # once: local .env + secrets
./scripts/staging-local.sh up        # antifraud → binance-sim → oracle → gateway
./scripts/staging-local.sh smoke     # smoke against http://127.0.0.1:8080
./scripts/staging-local.sh down      # stop processes
```

**With Docker** (same-origin Caddy on `:8080`):

```bash
# In deploy/staging/.env: STAGING_DOMAIN=:80, PUBLIC_BASE_URL=http://localhost:8080
./scripts/staging-local.sh up --docker
```

**Against Solana devnet** (Oracle queries public RPC; stack remains native local):

```bash
# deploy/staging/.env → SOLANA_RPC_URL=https://api.devnet.solana.com + wallet/mint with SPL
./scripts/staging-devnet.sh          # verify balance + restart Oracle + 3-rail smoke
./scripts/staging-devnet.sh --check  # devnet preflight only
```

---

## 3. Post-deploy verification

```bash
export STAGING_BASE_URL=https://staging.your-domain.com
export GATEWAY_TEST_API_KEY=sk_test_...   # value from .env
./scripts/smoke-staging.sh

export STAGING_VPS_IP=<VPS_IP>
./scripts/verify-staging-isolation.sh
```

Manual checks:

| Check | Command / action | Expected |
|-------|------------------|----------|
| Health | `curl -s $STAGING_BASE_URL/health` | JSON `status: ok` |
| Oracle not exposed | `curl -s --max-time 2 http://<VPS_IP>:8081/health` | timeout / connection refused |
| Checkout UI | Browser → `$STAGING_BASE_URL` | Form loads; bank checkout OK |
| Logs without PAN | `docker compose logs gateway oracle` | No full card numbers |

---

## 4. Operations

### Logs

```bash
cd deploy/staging
docker compose --env-file .env logs -f gateway oracle caddy
```

### Restart / rollback

```bash
# Soft restart
docker compose --env-file .env restart gateway oracle

# Rollback to previous image (manual tag recommended on deploy)
docker compose --env-file .env pull
docker compose --env-file .env up -d
```

### PostgreSQL backup

```bash
docker compose --env-file .env exec postgres \
  pg_dump -U pasarela pasarela_gateway > backup-gateway-$(date +%F).sql
docker compose --env-file .env exec postgres \
  pg_dump -U pasarela oracle > backup-oracle-$(date +%F).sql
```

---

## 5. Staging security

- Oracle **does not** publish ports in `docker-compose.yml`.
- `ORACLE_ALLOWED_CALLERS=0.0.0.0/0` staging QA only; restrict in production (Phase 8).
- Secrets in server `.env` — migrate to manager (Vault / SM) in Phase 8.
- Rotate keys before go-live.

---

## 6. Troubleshooting

| Symptom | Likely cause | Action |
|---------|--------------|--------|
| Caddy cannot obtain cert | DNS / port 80 | Verify A record; `docker compose logs caddy` |
| Gateway unhealthy | Oracle / Postgres | `docker compose logs oracle gateway` |
| Checkout 502 | Gateway down | `docker compose ps`; restart gateway |
| Solana smoke fails | Wallet without SPL | Review §2.2; try `--rail bank` first |
| Browser CORS | API on different origin | Use same-origin Caddy; or `GATEWAY_CORS_ORIGINS` |

---

## 7. Phase 7 gate

- [ ] Stack deployed on VPS with TLS
- [ ] `./scripts/smoke-staging.sh` green (3 rails)
- [ ] Oracle not accessible from Internet
- [ ] Runbook and rollback tested
- [ ] Explicit confirmation for Phase 8

See also: [Plan-de-Implementacion-en.md](./Plan-de-Implementacion-en.md) §11, [CI-en.md](./CI-en.md).
