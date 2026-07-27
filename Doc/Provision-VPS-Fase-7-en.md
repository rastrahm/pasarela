# VPS Provisioning — Phase 7 Block 2

> Operational checklist to go from repo artifacts to **real staging** on a VPS.

---

## Flow summary

```mermaid
flowchart LR
    A[New VPS] --> B[provision-vps.sh]
    B --> C[DNS A → IP]
    C --> D[git clone + generate-secrets]
    D --> E[edit .env]
    E --> F[deploy-staging.sh]
    F --> G[smoke + verify-isolation]
```

---

## 1. Choose VPS

| Provider | Minimum spec | Notes |
|----------|--------------|-------|
| Hetzner / DigitalOcean / Vultr / AWS Lightsail | 2 vCPU, 4 GB RAM, 40 GB SSD | Ubuntu 22.04 LTS |
| Region | Near QA team | Checkout latency |

Record: VPS **public IP**.

---

## 2. Provision server

SSH as root:

```bash
# From your machine (cloned repo):
scp deploy/staging/provision-vps.sh root@<VPS_IP>:/tmp/
ssh root@<VPS_IP> 'bash /tmp/provision-vps.sh'
```

What the script does:

- Installs Docker Engine + Compose plugin
- UFW: allows **22, 80, 443**; denies the rest
- Creates `deploy` user in `docker` group (optional)

```bash
ssh-copy-id deploy@<VPS_IP>
```

---

## 3. DNS and domain

1. Create **A** (or AAAA) record: `staging.your-domain.com` → `<VPS_IP>`
2. Wait for propagation (1–30 min)
3. Verify: `dig +short staging.your-domain.com`

---

## 4. Clone repo and secrets

As `deploy` user on VPS:

```bash
git clone <repo-url> ~/pasarela && cd ~/pasarela
./scripts/generate-staging-secrets.sh
chmod 600 deploy/staging/.env
```

Edit `deploy/staging/.env`:

| Variable | Value |
|----------|-------|
| `STAGING_DOMAIN` | `staging.your-domain.com` |
| `PUBLIC_BASE_URL` | `https://staging.your-domain.com` |
| `ACME_EMAIL` | valid email for Let's Encrypt |
| `SOLANA_WALLET_PUBKEY` | devnet wallet with SPL |
| `SOLANA_TOKEN_MINT` | devnet mint used |

Validate:

```bash
./scripts/check-staging-env.sh --preflight
./scripts/setup-solana-devnet-staging.sh   # optional, Solana rail
```

---

## 5. First deploy

```bash
./scripts/deploy-staging.sh
# First time: Rust build ~15–25 min on modest VPS
```

Verify on VPS:

```bash
cd deploy/staging
docker compose --env-file .env ps
curl -s https://staging.your-domain.com/health
```

---

## 6. Verification from your machine (Block 2–3 gate)

```bash
export STAGING_BASE_URL=https://staging.your-domain.com
export GATEWAY_TEST_API_KEY=sk_test_...   # from deploy/staging/.env on VPS
./scripts/smoke-staging.sh

export STAGING_VPS_IP=<VPS_IP>
./scripts/verify-staging-isolation.sh
```

| Check | Expected |
|-------|----------|
| 3-rail smoke | 3/3 OK |
| Ports 8080–8083, 5432 | Closed from Internet |
| TLS | Valid certificate in browser |
| Oracle logs | No full PAN |

---

## 7. Subsequent deploys

### Manual (SSH)

```bash
ssh deploy@<VPS_IP> 'cd ~/pasarela && git pull && ./scripts/deploy-staging.sh'
```

### GitHub Actions (optional)

Configure secrets: `STAGING_SSH_HOST`, `STAGING_SSH_USER`, `STAGING_SSH_KEY`.

Actions → **Deploy Staging** → Run workflow → `main` branch, `tls` profile.

---

## 8. Quick rollback

```bash
cd ~/pasarela
git checkout <previous-commit>
./scripts/deploy-staging.sh --no-smoke
```

DB backup before major changes:

```bash
cd deploy/staging
docker compose --env-file .env exec postgres \
  pg_dump -U pasarela pasarela_gateway > ~/backup-gateway.sql
```

---

## 9. Block 2 gate

- [ ] VPS provisioned (`provision-vps.sh`)
- [ ] DNS pointing to VPS
- [ ] `deploy/staging/.env` with real secrets (not placeholders)
- [ ] `deploy-staging.sh` successful + Caddy TLS active
- [ ] Bank rail smoke green from Internet

Block 3 gate (next): 3-rail smoke + Oracle isolation verified.

See [Runbook-Staging-en.md](./Runbook-Staging-en.md), [Plan-de-Implementacion-en.md](./Plan-de-Implementacion-en.md) §11.
