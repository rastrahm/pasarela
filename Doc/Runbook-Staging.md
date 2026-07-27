> **Documentation / Documentación:** [Español (es)](Runbook-Staging-es.md) · [English (en)](Runbook-Staging-en.md)
>
# Runbook — Staging (Fase 7)

> Entorno pre-producción: **1 VPS + Docker Compose + Caddy TLS**.  
> Oracle en red interna (sin puerto público). Frontend y API same-origin vía Caddy.

---

## 1. Prerrequisitos

| Requisito | Detalle |
|-----------|---------|
| VPS | 2 vCPU, 4 GB RAM mínimo; Ubuntu 22.04+ o Debian 12 |
| Docker | Engine 24+ y Compose v2 |
| Dominio | DNS `A` / `AAAA` apuntando al VPS (TLS Let's Encrypt) |
| Puertos | 80 y 443 abiertos hacia Caddy |
| Devnet | `https://api.devnet.solana.com` alcanzable desde el VPS |

---

## 2. Primer deploy

Guía paso a paso completa: **[Provision-VPS-Fase-7.md](./Provision-VPS-Fase-7.md)** (Bloque 2).

### 2.1 Clonar y configurar secretos

```bash
git clone <repo-url> pasarela && cd pasarela
./scripts/generate-staging-secrets.sh
# Editar deploy/staging/.env — dominio, devnet, ACME_EMAIL
./scripts/check-staging-env.sh --preflight
openssl rand -hex 32   # solo si regenerás secretos a mano
```

Variables críticas:

- `STAGING_DOMAIN` — dominio público (ej. `staging.pasarela.ejemplo.com`)
- `PUBLIC_BASE_URL` — `https://<STAGING_DOMAIN>` (build del frontend)
- `POSTGRES_PASSWORD`, `ORACLE_API_KEY`, claves compartidas
- `SOLANA_WALLET_PUBKEY` + `SOLANA_TOKEN_MINT` — wallet con SPL en devnet

Programa Anchor en devnet: `programs/payment-settlement/deploy/devnet.json`.

### 2.2 Solana devnet (riel `solana_wallet`)

1. Crear wallet devnet y fondear SOL (`solana airdrop 2`).
2. Crear o usar mint SPL de prueba; acreditar tokens a la wallet.
3. Copiar pubkey y mint a `deploy/staging/.env`.
4. Smoke riel Solana: `./scripts/smoke-staging.sh --rail solana`.

### 2.3 Levantar stack (TLS)

```bash
./scripts/deploy-staging.sh
# equivalente manual:
# cd deploy/staging && docker compose --env-file .env --profile tls up -d --build
docker compose --env-file deploy/staging/.env -f deploy/staging/docker-compose.yml ps
```

Servicios internos (sin puerto en host): `oracle`, `antifraud`, `binance-sim`, `postgres`, `gateway`.  
Único punto de entrada: **Caddy** (:80 / :443).

### 2.4 QA local sin dominio (perfil `local`)

**Sin Docker** (recomendado en dev — usa el stack `cargo` existente):

```bash
./scripts/staging-local.sh init    # una vez: .env local + secretos
./scripts/staging-local.sh up        # antifraud → binance-sim → oracle → gateway
./scripts/staging-local.sh smoke     # smoke contra http://127.0.0.1:8080
./scripts/staging-local.sh down      # detener procesos
```

**Con Docker** (same-origin Caddy en `:8080`):

```bash
# En deploy/staging/.env: STAGING_DOMAIN=:80, PUBLIC_BASE_URL=http://localhost:8080
./scripts/staging-local.sh up --docker
```

**Contra Solana devnet** (Oracle consulta RPC público; stack sigue native local):

```bash
# deploy/staging/.env → SOLANA_RPC_URL=https://api.devnet.solana.com + wallet/mint con SPL
./scripts/staging-devnet.sh          # verifica saldo + reinicia Oracle + smoke 3 rieles
./scripts/staging-devnet.sh --check  # solo preflight devnet
```

---

## 3. Verificación post-deploy

```bash
export STAGING_BASE_URL=https://staging.tu-dominio.com
export GATEWAY_TEST_API_KEY=sk_test_...   # valor de .env
./scripts/smoke-staging.sh

export STAGING_VPS_IP=<IP_DEL_VPS>
./scripts/verify-staging-isolation.sh
```

Checks manuales:

| Check | Comando / acción | Esperado |
|-------|------------------|----------|
| Health | `curl -s $STAGING_BASE_URL/health` | JSON `status: ok` |
| Oracle no expuesto | `curl -s --max-time 2 http://<VPS_IP>:8081/health` | timeout / connection refused |
| UI checkout | Navegador → `$STAGING_BASE_URL` | Formulario carga; checkout banco OK |
| Logs sin PAN | `docker compose logs gateway oracle` | Sin números de tarjeta completos |

---

## 4. Operación

### Logs

```bash
cd deploy/staging
docker compose --env-file .env logs -f gateway oracle caddy
```

### Reinicio / rollback

```bash
# Reinicio suave
docker compose --env-file .env restart gateway oracle

# Rollback a imagen anterior (tag manual recomendado en deploy)
docker compose --env-file .env pull
docker compose --env-file .env up -d
```

### Backup PostgreSQL

```bash
docker compose --env-file .env exec postgres \
  pg_dump -U pasarela pasarela_gateway > backup-gateway-$(date +%F).sql
docker compose --env-file .env exec postgres \
  pg_dump -U pasarela oracle > backup-oracle-$(date +%F).sql
```

---

## 5. Seguridad staging

- Oracle **no** publica puertos en `docker-compose.yml`.
- `ORACLE_ALLOWED_CALLERS=0.0.0.0/0` solo QA staging; producción restringir (Fase 8).
- Secretos en `.env` del servidor — migrar a gestor (Vault / SM) en Fase 8.
- Rotar claves antes de go-live.

---

## 6. Troubleshooting

| Síntoma | Causa probable | Acción |
|---------|----------------|--------|
| Caddy no obtiene cert | DNS / puerto 80 | Verificar registro A; `docker compose logs caddy` |
| Gateway unhealthy | Oracle / Postgres | `docker compose logs oracle gateway` |
| Checkout 502 | Gateway caído | `docker compose ps`; reiniciar gateway |
| Solana smoke falla | Wallet sin SPL | Revisar §2.2; probar `--rail bank` primero |
| CORS en navegador | API en otro origen | Usar same-origin Caddy; o `GATEWAY_CORS_ORIGINS` |

---

## 7. Gate Fase 7

- [ ] Stack desplegado en VPS con TLS
- [ ] `./scripts/smoke-staging.sh` verde (3 rieles)
- [ ] Oracle no accesible desde Internet
- [ ] Runbook y rollback probados
- [ ] Confirmación explícita para Fase 8

Ver también: [Plan-de-Implementacion.md](./Plan-de-Implementacion.md) §11, [CI.md](./CI.md).
