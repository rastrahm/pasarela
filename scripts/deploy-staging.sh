#!/usr/bin/env bash
# Deploy del stack staging en el VPS (Fase 7 Bloque 2).
#
# Ejecutar en el servidor (repo clonado):
#   ./scripts/deploy-staging.sh
#   ./scripts/deploy-staging.sh --profile local
#   ./scripts/deploy-staging.sh --no-smoke
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT/deploy/staging/.env"
PROFILE="tls"
RUN_SMOKE=true
COMPOSE=(docker compose --env-file "$ENV_FILE")

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:?}"
      shift 2
      ;;
    --no-smoke)
      RUN_SMOKE=false
      shift
      ;;
    -h|--help)
      sed -n '1,10p' "$0"
      exit 0
      ;;
    *)
      echo "Argumento desconocido: $1" >&2
      exit 2
      ;;
  esac
done

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: Docker no instalado. Ejecutá deploy/staging/provision-vps.sh en el VPS." >&2
  exit 1
fi

echo "=== deploy-staging — Pasarela Multi-Rail ==="
echo "Perfil: ${PROFILE}"
echo

if [[ "$PROFILE" == "local" ]]; then
  "$ROOT/scripts/check-staging-env.sh" --local
else
  "$ROOT/scripts/check-staging-env.sh" --preflight
fi

cd "$ROOT/deploy/staging"

echo "Build y arranque (puede tardar varios minutos la primera vez)..."
"${COMPOSE[@]}" --profile "$PROFILE" up -d --build

echo
echo "Esperando Gateway healthy..."
ready=false
for _ in $(seq 1 36); do
  if "${COMPOSE[@]}" --profile "$PROFILE" exec -T gateway \
    curl -sf http://127.0.0.1:8080/health >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 5
done

if [[ "$ready" != true ]]; then
  echo "ERROR: Gateway no healthy tras 3 min — logs:" >&2
  "${COMPOSE[@]}" --profile "$PROFILE" ps
  "${COMPOSE[@]}" --profile "$PROFILE" logs --tail=50 gateway oracle postgres caddy
  exit 1
fi
info_msg() { printf '  ✓ %s\n' "$1"; }
info_msg "Gateway responde /health"

echo
"${COMPOSE[@]}" --profile "$PROFILE" ps
echo

# shellcheck disable=SC1090
PUBLIC_BASE_URL="$(grep -E '^PUBLIC_BASE_URL=' "$ENV_FILE" | head -1 | cut -d= -f2- | tr -d '"')"
GATEWAY_TEST_API_KEY="$(grep -E '^GATEWAY_TEST_API_KEY=' "$ENV_FILE" | head -1 | cut -d= -f2- | tr -d '"')"

if [[ "$RUN_SMOKE" == true ]]; then
  echo "Smoke post-deploy..."
  export STAGING_BASE_URL="${PUBLIC_BASE_URL}"
  export GATEWAY_TEST_API_KEY
  if [[ "$PROFILE" == "local" ]]; then
    "$ROOT/scripts/smoke-staging.sh" --rail bank
    echo
    echo "Smoke 3 rieles: ./scripts/staging-local.sh smoke"
  else
    "$ROOT/scripts/smoke-staging.sh" --rail bank
    echo
    echo "Smoke completo (3 rieles): ejecutá desde tu máquina con acceso HTTPS:"
    echo "  export STAGING_BASE_URL=${PUBLIC_BASE_URL}"
    echo "  export GATEWAY_TEST_API_KEY=<de .env>"
    echo "  ./scripts/smoke-staging.sh"
  fi
fi

echo
echo "Deploy staging OK — perfil ${PROFILE}"
