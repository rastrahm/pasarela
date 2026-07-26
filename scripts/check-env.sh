#!/usr/bin/env bash
# Valida sincronización de .env del stack local (Fase 6.7 / DT-P0-03, DT-P0-04).
#
# Uso:
#   ./scripts/check-env.sh           # comparar pares de variables en archivos .env
#   ./scripts/check-env.sh --live    # además, healthchecks HTTP de servicios
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIVE=false
ERRORS=0
WARNINGS=0

if [[ "${1:-}" == "--live" ]]; then
  LIVE=true
fi

info() { printf '  ✓ %s\n' "$1"; }
warn() { printf '  ⚠ %s\n' "$1"; WARNINGS=$((WARNINGS + 1)); }
fail() { printf '  ✗ %s\n' "$1"; ERRORS=$((ERRORS + 1)); }

# Lee KEY=valor de un archivo .env (ignora comentarios y líneas vacías).
get_env() {
  local file="$1" key="$2"
  if [[ ! -f "$file" ]]; then
    echo ""
    return 0
  fi
  local line
  line="$(grep -E "^${key}=" "$file" 2>/dev/null | head -1 || true)"
  if [[ -z "$line" ]]; then
    echo ""
    return 0
  fi
  local value="${line#*=}"
  value="${value#\"}"; value="${value%\"}"
  value="${value#\'}"; value="${value%\'}"
  printf '%s' "$value"
}

require_file() {
  local file="$1" label="$2"
  if [[ ! -f "$file" ]]; then
    warn "Falta ${label}: ${file#"$ROOT"/} (copiá desde .env.example)"
    return 1
  fi
  info "Encontrado ${label}: ${file#"$ROOT"/}"
  return 0
}

compare_env() {
  local label="$1" key_a="$2" file_a="$3" key_b="$4" file_b="$5"
  local val_a val_b
  val_a="$(get_env "$file_a" "$key_a")"
  val_b="$(get_env "$file_b" "$key_b")"

  if [[ -z "$val_a" && -z "$val_b" ]]; then
    warn "${label}: ambos vacíos (${key_a} / ${key_b})"
    return 0
  fi
  if [[ -z "$val_a" ]]; then
    fail "${label}: falta ${key_a} en ${file_a#"$ROOT"/}"
    return 0
  fi
  if [[ -z "$val_b" ]]; then
    fail "${label}: falta ${key_b} en ${file_b#"$ROOT"/}"
    return 0
  fi
  if [[ "$val_a" != "$val_b" ]]; then
    fail "${label}: no coinciden (${key_a} ≠ ${key_b})"
    return 0
  fi
  info "${label}: sincronizado"
}

check_url() {
  local name="$1" url="$2"
  if curl -sf --max-time 2 "$url" >/dev/null 2>&1; then
    info "${name} responde en ${url}"
  else
    fail "${name} no responde en ${url}"
  fi
}

GATEWAY_ENV="$ROOT/crates/api-gateway/.env"
ORACLE_ENV="$ROOT/oracle/.env"
ANTIFRAUD_ENV="$ROOT/antifraud/.env"
BINANCE_ENV="$ROOT/binance-sim/.env"
FRONTEND_ENV="$ROOT/frontend/.env.local"
FRONTEND_FALLBACK="$ROOT/frontend/.env.example"

echo "=== check-env — Pasarela Multi-Rail ==="
echo "Raíz: $ROOT"
echo

echo "Archivos de entorno:"
require_file "$GATEWAY_ENV" "Gateway" || true
require_file "$ORACLE_ENV" "Oracle" || true
require_file "$ANTIFRAUD_ENV" "Antifraude" || true
require_file "$BINANCE_ENV" "Binance sim" || true
if [[ -f "$FRONTEND_ENV" ]]; then
  info "Encontrado Frontend: frontend/.env.local"
  FRONTEND_SRC="$FRONTEND_ENV"
else
  warn "Falta frontend/.env.local — usando .env.example como referencia"
  FRONTEND_SRC="$FRONTEND_FALLBACK"
fi
echo

echo "Pares sincronizados:"
compare_env "Gateway ↔ Oracle" \
  "ORACLE_API_KEY" "$GATEWAY_ENV" \
  "ORACLE_API_KEY" "$ORACLE_ENV"

compare_env "Gateway ↔ Frontend (API key comercio)" \
  "GATEWAY_TEST_API_KEY" "$GATEWAY_ENV" \
  "VITE_GATEWAY_API_KEY" "$FRONTEND_SRC"

compare_env "Oracle ↔ Antifraude" \
  "ANTIFRAUD_API_KEY" "$ORACLE_ENV" \
  "ANTIFRAUD_API_KEY" "$ANTIFRAUD_ENV"

compare_env "Oracle ↔ Binance sim" \
  "BINANCE_CEX_API_KEY" "$ORACLE_ENV" \
  "BINANCE_CEX_API_KEY" "$BINANCE_ENV"
echo

echo "URLs esperadas:"
oracle_url="$(get_env "$GATEWAY_ENV" "ORACLE_BASE_URL")"
oracle_url="${oracle_url:-http://127.0.0.1:8081}"
antifraud_url="$(get_env "$ORACLE_ENV" "ANTIFRAUD_BASE_URL")"
antifraud_url="${antifraud_url:-http://127.0.0.1:8082}"
binance_url="$(get_env "$ORACLE_ENV" "BINANCE_CEX_BASE_URL")"
binance_url="${binance_url:-http://127.0.0.1:8083}"
gateway_port="$(get_env "$GATEWAY_ENV" "GATEWAY_PORT")"
gateway_port="${gateway_port:-8080}"
frontend_url="$(get_env "$FRONTEND_SRC" "VITE_API_BASE_URL")"
frontend_url="${frontend_url:-http://127.0.0.1:8080}"

if [[ "$oracle_url" == "http://127.0.0.1:8081" || "$oracle_url" == "http://localhost:8081" ]]; then
  info "ORACLE_BASE_URL → ${oracle_url}"
else
  warn "ORACLE_BASE_URL no es el default local: ${oracle_url}"
fi

if [[ "$frontend_url" == "http://127.0.0.1:${gateway_port}" || "$frontend_url" == "http://localhost:${gateway_port}" ]]; then
  info "VITE_API_BASE_URL apunta al Gateway (${frontend_url})"
else
  warn "VITE_API_BASE_URL (${frontend_url}) puede no coincidir con GATEWAY_PORT=${gateway_port}"
fi
echo

if [[ "$LIVE" == true ]]; then
  echo "Healthchecks (--live):"
  check_url "Antifraude" "${antifraud_url%/}/health"
  check_url "Binance sim" "${binance_url%/}/health"
  check_url "Oracle" "${oracle_url%/}/health"
  check_url "Gateway" "http://127.0.0.1:${gateway_port}/health"
  echo
fi

echo "=== Resumen ==="
echo "  Errores:   ${ERRORS}"
echo "  Advertencias: ${WARNINGS}"

if [[ "$ERRORS" -gt 0 ]]; then
  echo
  echo "Revisá Doc/Runbook-Desarrollo.md §2.2 y §7."
  exit 1
fi

echo "OK — entorno coherente para desarrollo local."
exit 0
