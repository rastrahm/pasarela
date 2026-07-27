#!/usr/bin/env bash
# Smoke tests post-deploy en staging (Fase 7.11).
#
# Uso:
#   export STAGING_BASE_URL=https://staging.ejemplo.com
#   export GATEWAY_TEST_API_KEY=sk_test_...
#   ./scripts/smoke-staging.sh
#
#   ./scripts/smoke-staging.sh --rail bank|binance|solana|all
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RAIL="${SMOKE_RAIL:-all}"
BASE_URL="${STAGING_BASE_URL:-${PUBLIC_BASE_URL:-}}"
API_KEY="${GATEWAY_TEST_API_KEY:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rail)
      RAIL="${2:?}"
      shift 2
      ;;
    --url)
      BASE_URL="${2:?}"
      shift 2
      ;;
    -h|--help)
      sed -n '1,12p' "$0"
      exit 0
      ;;
    *)
      echo "Argumento desconocido: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$BASE_URL" ]]; then
  echo "ERROR: definí STAGING_BASE_URL o PUBLIC_BASE_URL (sin barra final)." >&2
  exit 1
fi

if [[ -z "$API_KEY" ]]; then
  echo "ERROR: definí GATEWAY_TEST_API_KEY." >&2
  exit 1
fi

BASE_URL="${BASE_URL%/}"
FIXTURES="$ROOT/scripts/fixtures"
PASS=0
FAIL=0

info() { printf '  ✓ %s\n' "$1"; PASS=$((PASS + 1)); }
die() { printf '  ✗ %s\n' "$1"; FAIL=$((FAIL + 1)); return 1; }

check_health() {
  local body
  if ! body="$(curl -sf --max-time 10 "${BASE_URL}/health")"; then
    die "GET /health no respondió"
    return 0
  fi
  if echo "$body" | grep -q '"status"'; then
    info "GET /health OK"
  else
    die "GET /health respuesta inesperada: ${body}"
  fi
}

checkout_rail() {
  local name="$1" fixture="$2"
  local idem body status

  idem="smoke-$(uuidgen 2>/dev/null || echo "${RANDOM}-${RANDOM}")"
  fixture_path="${FIXTURES}/${fixture}"

  if [[ ! -f "$fixture_path" ]]; then
    die "fixture ausente: ${fixture}"
    return 0
  fi

  status="$(curl -s -o /tmp/smoke-checkout.json -w '%{http_code}' --max-time 30 \
    -X POST "${BASE_URL}/api/v1/checkout" \
    -H "Authorization: Bearer ${API_KEY}" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: ${idem}" \
    -H "X-Forwarded-For: 127.0.0.1" \
    -d @"${fixture_path}")"

  body="$(cat /tmp/smoke-checkout.json)"

  if [[ "$status" != "200" ]]; then
    die "checkout ${name}: HTTP ${status} — ${body}"
    return 0
  fi

  if echo "$body" | grep -q '"transaction_id"'; then
    info "checkout ${name}: HTTP 200 + transaction_id"
  else
    die "checkout ${name}: respuesta sin transaction_id — ${body}"
  fi
}

echo "=== smoke-staging — Pasarela Multi-Rail ==="
echo "URL:  ${BASE_URL}"
echo "Riel: ${RAIL}"
echo

echo "Health:"
check_health || true
echo

echo "Checkout:"
case "$RAIL" in
  bank)
    checkout_rail "traditional_bank" "checkout-bank.json" || true
    ;;
  binance)
    checkout_rail "binance_cex" "checkout-binance.json" || true
    ;;
  solana)
    checkout_rail "solana_wallet" "checkout-solana.json" || true
    ;;
  all)
    checkout_rail "traditional_bank" "checkout-bank.json" || true
    checkout_rail "binance_cex" "checkout-binance.json" || true
    checkout_rail "solana_wallet" "checkout-solana.json" || true
    ;;
  *)
    echo "Riel desconocido: ${RAIL} (bank|binance|solana|all)" >&2
    exit 2
    ;;
esac

echo
echo "=== Resumen ==="
echo "  OK:    ${PASS}"
echo "  Fallos: ${FAIL}"

if [[ "$FAIL" -gt 0 ]]; then
  echo
  echo "Revisá Doc/Runbook-Staging.md §6 (Solana devnet) si falló solana_wallet."
  exit 1
fi

echo "Smoke staging OK."
exit 0
