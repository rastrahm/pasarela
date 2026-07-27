#!/usr/bin/env bash
# Genera deploy/staging/.env desde .env.example con secretos aleatorios (Fase 7.4).
#
# Uso:
#   ./scripts/generate-staging-secrets.sh
#   ./scripts/generate-staging-secrets.sh --force   # sobrescribe .env existente
#
# Después editá STAGING_DOMAIN, PUBLIC_BASE_URL, ACME_EMAIL y wallet devnet.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLE="$ROOT/deploy/staging/.env.example"
OUT="$ROOT/deploy/staging/.env"
FORCE=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force) FORCE=true; shift ;;
    -h|--help)
      sed -n '1,8p' "$0"
      exit 0
      ;;
    *) echo "Argumento desconocido: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -f "$EXAMPLE" ]]; then
  echo "ERROR: no existe $EXAMPLE" >&2
  exit 1
fi

if [[ -f "$OUT" && "$FORCE" != true ]]; then
  echo "ERROR: $OUT ya existe. Usá --force para sobrescribir." >&2
  exit 1
fi

rand_hex() { openssl rand -hex 32; }
test_api_key() { printf 'sk_test_%s' "$(openssl rand -hex 20)"; }

PG_PASS="$(rand_hex)"
ORACLE_KEY="$(rand_hex)"
ANTIFRAUD_KEY="$(rand_hex)"
BINANCE_KEY="$(rand_hex)"
GATEWAY_KEY="$(test_api_key)"

cp "$EXAMPLE" "$OUT"
chmod 600 "$OUT"

replace() {
  local key="$1" value="$2"
  sed -i "s|^${key}=.*|${key}=${value}|" "$OUT"
}

replace POSTGRES_PASSWORD "$PG_PASS"
replace ORACLE_API_KEY "$ORACLE_KEY"
replace ANTIFRAUD_API_KEY "$ANTIFRAUD_KEY"
replace BINANCE_CEX_API_KEY "$BINANCE_KEY"
replace GATEWAY_TEST_API_KEY "$GATEWAY_KEY"

echo "=== generate-staging-secrets OK ==="
echo "Archivo: deploy/staging/.env (permisos 600)"
echo
echo "Editá antes del deploy:"
echo "  STAGING_DOMAIN, PUBLIC_BASE_URL, ACME_EMAIL"
echo "  SOLANA_WALLET_PUBKEY, SOLANA_TOKEN_MINT (devnet con SPL)"
echo
echo "Validar: ./scripts/check-staging-env.sh"
