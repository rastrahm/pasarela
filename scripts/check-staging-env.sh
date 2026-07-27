#!/usr/bin/env bash
# Valida deploy/staging/.env antes del deploy (Fase 7 Bloque 2).
#
# Uso:
#   ./scripts/check-staging-env.sh
#   ./scripts/check-staging-env.sh --local          # perfil native/Docker local
#   ./scripts/check-staging-env.sh --preflight      # + RPC devnet y DNS
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT/deploy/staging/.env"
PREFLIGHT=false
LOCAL=false
ERRORS=0
WARNINGS=0

for arg in "$@"; do
  case "$arg" in
    --preflight) PREFLIGHT=true ;;
    --local) LOCAL=true ;;
    -h|--help)
      sed -n '1,8p' "$0"
      exit 0
      ;;
    *)
      echo "Uso: $0 [--local] [--preflight]" >&2
      exit 2
      ;;
  esac
done

info() { printf '  ✓ %s\n' "$1"; }
warn() { printf '  ⚠ %s\n' "$1"; WARNINGS=$((WARNINGS + 1)); }
fail() { printf '  ✗ %s\n' "$1"; ERRORS=$((ERRORS + 1)); }

get_env() {
  local key="$1"
  grep -E "^${key}=" "$ENV_FILE" 2>/dev/null | head -1 | cut -d= -f2- \
    | sed 's/^["'\''"]//;s/["'\''"]$//'
}

is_placeholder() {
  local value="$1"
  [[ -z "$value" ]] && return 0
  [[ "$value" == *change-me* ]] && return 0
  [[ "$value" == *change_me* ]] && return 0
  [[ "$value" == *ejemplo.com* ]] && return 0
  [[ "$value" == DemoWallet* ]] && return 0
  return 1
}

echo "=== check-staging-env — Pasarela Multi-Rail ==="
echo "Archivo: deploy/staging/.env"
echo

if [[ ! -f "$ENV_FILE" ]]; then
  fail "Falta deploy/staging/.env — ejecutá ./scripts/generate-staging-secrets.sh"
  exit 1
fi

if [[ "$(stat -c '%a' "$ENV_FILE" 2>/dev/null || stat -f '%OLp' "$ENV_FILE")" != "600" ]]; then
  warn "Permisos de .env no son 600 (recomendado: chmod 600 deploy/staging/.env)"
else
  info "Permisos .env: 600"
fi

REQUIRED_KEYS=(
  STAGING_DOMAIN
  PUBLIC_BASE_URL
  ACME_EMAIL
  POSTGRES_PASSWORD
  ORACLE_API_KEY
  ANTIFRAUD_API_KEY
  BINANCE_CEX_API_KEY
  GATEWAY_TEST_API_KEY
  SOLANA_WALLET_PUBKEY
  SOLANA_TOKEN_MINT
)

echo "Variables obligatorias:"
for key in "${REQUIRED_KEYS[@]}"; do
  val="$(get_env "$key")"
  if [[ -z "$val" ]]; then
    fail "${key} vacía"
  elif is_placeholder "$val"; then
    if [[ "$LOCAL" == true && ( "$key" == SOLANA_WALLET_PUBKEY || "$key" == SOLANA_TOKEN_MINT || "$key" == ACME_EMAIL ) ]]; then
      warn "${key} placeholder (${val}) — OK en local; riel Solana puede fallar"
    else
      fail "${key} sigue con valor placeholder (${val})"
    fi
  else
    info "${key} definida"
  fi
done
echo

STAGING_DOMAIN="$(get_env STAGING_DOMAIN)"
PUBLIC_BASE_URL="$(get_env PUBLIC_BASE_URL)"
GATEWAY_KEY="$(get_env GATEWAY_TEST_API_KEY)"

echo "Coherencia:"
if [[ ${#GATEWAY_KEY} -lt 24 ]]; then
  fail "GATEWAY_TEST_API_KEY demasiado corta (mín. 24 caracteres)"
else
  info "GATEWAY_TEST_API_KEY longitud OK"
fi

if [[ "$LOCAL" == true && "$STAGING_DOMAIN" != ":80" ]]; then
  warn "Modo --local: STAGING_DOMAIN debería ser :80 (actual: ${STAGING_DOMAIN})"
fi

if [[ "$STAGING_DOMAIN" == ":80" ]]; then
  if [[ "$PUBLIC_BASE_URL" != http://localhost* && "$PUBLIC_BASE_URL" != http://127.0.0.1* ]]; then
    warn "Perfil local: PUBLIC_BASE_URL debería ser http://localhost:8080 (actual: ${PUBLIC_BASE_URL})"
  else
    info "Perfil local: dominio :80 + URL local"
  fi
elif [[ "$PUBLIC_BASE_URL" == "https://${STAGING_DOMAIN}" ]]; then
  info "PUBLIC_BASE_URL coincide con STAGING_DOMAIN (TLS)"
elif [[ "$PUBLIC_BASE_URL" == "http://${STAGING_DOMAIN}" ]]; then
  warn "PUBLIC_BASE_URL es HTTP — Caddy usará TLS si STAGING_DOMAIN es FQDN"
else
  warn "PUBLIC_BASE_URL (${PUBLIC_BASE_URL}) no coincide con https://${STAGING_DOMAIN}"
fi
echo

if [[ "$PREFLIGHT" == true || ( "$LOCAL" == true && "$PREFLIGHT" == false ) ]]; then
  echo "Preflight externo:"
  SOLANA_RPC="$(get_env SOLANA_RPC_URL)"
  SOLANA_RPC="${SOLANA_RPC:-https://api.devnet.solana.com}"

  if curl -sf --max-time 8 -X POST "$SOLANA_RPC" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' | grep -q '"result"'; then
    info "Solana RPC responde (${SOLANA_RPC})"
  else
    if [[ "$LOCAL" == true && "$SOLANA_RPC" == http://127.0.0.1* ]]; then
      warn "Validador local no responde — levantalo si probás riel Solana"
    else
      fail "Solana RPC no responde (${SOLANA_RPC})"
    fi
  fi

  if [[ "$PREFLIGHT" == true && "$STAGING_DOMAIN" != ":80" ]]; then
    if command -v dig >/dev/null 2>&1; then
      if dig +short "$STAGING_DOMAIN" A | grep -qE '^[0-9.]+$'; then
        info "DNS A record para ${STAGING_DOMAIN}"
      else
        warn "Sin registro A para ${STAGING_DOMAIN} — TLS Let's Encrypt fallará"
      fi
    else
      warn "dig no disponible — verificá DNS manualmente"
    fi
  fi
  echo
fi

echo "=== Resumen ==="
echo "  Errores:      ${ERRORS}"
echo "  Advertencias: ${WARNINGS}"

if [[ "$ERRORS" -gt 0 ]]; then
  echo
  echo "Ver Doc/Provision-VPS-Fase-7.md y Doc/Runbook-Staging.md"
  exit 1
fi

echo "OK — listo para deploy ($( [[ "$LOCAL" == true ]] && echo staging-local / deploy-staging --profile local || echo deploy-staging ))"
exit 0
