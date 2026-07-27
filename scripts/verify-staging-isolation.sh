#!/usr/bin/env bash
# Verifica que servicios internos NO sean accesibles desde Internet (Fase 7.2 / gate).
#
# Uso:
#   STAGING_VPS_IP=203.0.113.10 ./scripts/verify-staging-isolation.sh
#   ./scripts/verify-staging-isolation.sh 203.0.113.10
set -euo pipefail

VPS_IP="${STAGING_VPS_IP:-${1:-}}"
PORTS=(8080 8081 8082 8083 5432)
PASS=0
FAIL=0

if [[ -z "$VPS_IP" ]]; then
  echo "Uso: STAGING_VPS_IP=<ip> $0" >&2
  echo "     $0 <ip>" >&2
  exit 2
fi

info() { printf '  ✓ %s\n' "$1"; PASS=$((PASS + 1)); }
bad() { printf '  ✗ %s\n' "$1"; FAIL=$((FAIL + 1)); }

echo "=== verify-staging-isolation ==="
echo "VPS: ${VPS_IP}"
echo "Puertos internos que NO deben responder:"
echo

for port in "${PORTS[@]}"; do
  if timeout 3 bash -c "echo >/dev/tcp/${VPS_IP}/${port}" 2>/dev/null; then
    bad "Puerto ${port} ABIERTO — riesgo de exposición"
  else
    info "Puerto ${port} cerrado / filtrado"
  fi
done

echo
echo "=== Resumen ==="
echo "  OK:     ${PASS}"
echo "  Fallos: ${FAIL}"

if [[ "$FAIL" -gt 0 ]]; then
  echo
  echo "Revisá docker-compose.yml (sin ports en oracle/gateway/postgres)."
  exit 1
fi

echo "Aislamiento OK — solo Caddy (:80/:443) debe ser público."
exit 0
