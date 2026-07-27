#!/usr/bin/env bash
# Staging perfil LOCAL — native (cargo) o Docker Compose.
#
# Uso:
#   ./scripts/staging-local.sh init          # .env local desde plantilla + secretos
#   ./scripts/staging-local.sh up            # levanta stack native
#   ./scripts/staging-local.sh up --docker   # levanta stack Docker (perfil local)
#   ./scripts/staging-local.sh smoke         # smoke vía PUBLIC_BASE_URL
#   ./scripts/staging-local.sh down          # detiene procesos
#   ./scripts/staging-local.sh status
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGING_ENV="$ROOT/deploy/staging/.env"
RUN_DIR="/tmp/pasarela-staging"
PID_DIR="$RUN_DIR/pids"
LOG_DIR="$RUN_DIR/logs"
USE_DOCKER=false
CMD="${1:-}"

shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --docker) USE_DOCKER=true; shift ;;
    *) break ;;
  esac
done

mkdir -p "$PID_DIR" "$LOG_DIR"

health() {
  curl -sf --max-time 2 "$1" >/dev/null 2>&1
}

wait_health() {
  local name="$1" url="$2" tries="${3:-60}"
  for _ in $(seq 1 "$tries"); do
    if health "$url"; then
      printf '  ✓ %s\n' "$name"
      return 0
    fi
    sleep 2
  done
  printf '  ✗ %s no respondió en %s\n' "$name" "$url" >&2
  return 1
}

start_native() {
  local svc="$1" dir="$2" port="$3"
  local pid_file="$PID_DIR/${svc}.pid"
  if [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null; then
    printf '  · %s ya corre (pid %s)\n' "$svc" "$(cat "$pid_file")"
    return 0
  fi
  (cd "$ROOT/$dir" && nohup cargo run -q >"$LOG_DIR/${svc}.log" 2>&1 & echo $! >"$pid_file")
  wait_health "$svc" "http://127.0.0.1:${port}/health" 90
}

stop_native() {
  for pid_file in "$PID_DIR"/*.pid; do
    [[ -f "$pid_file" ]] || continue
    pid="$(cat "$pid_file")"
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      printf '  detenido pid %s (%s)\n' "$pid" "$(basename "$pid_file" .pid)"
    fi
    rm -f "$pid_file"
  done
}

cmd_init() {
  if [[ ! -f "$STAGING_ENV" ]]; then
    cp "$ROOT/deploy/staging/.env.local.example" "$STAGING_ENV"
    "$ROOT/scripts/generate-staging-secrets.sh" --force
  fi
  sed -i 's|^STAGING_DOMAIN=.*|STAGING_DOMAIN=:80|' "$STAGING_ENV"
  sed -i 's|^PUBLIC_BASE_URL=.*|PUBLIC_BASE_URL=http://localhost:8080|' "$STAGING_ENV"
  grep -q '^CADDY_HTTP_PORT=' "$STAGING_ENV" \
    && sed -i 's|^CADDY_HTTP_PORT=.*|CADDY_HTTP_PORT=8080|' "$STAGING_ENV" \
    || echo 'CADDY_HTTP_PORT=8080' >> "$STAGING_ENV"
  sed -i 's|^ACME_EMAIL=.*|ACME_EMAIL=local@localhost|' "$STAGING_ENV"

  # Reutilizar Solana local si ya estaba configurado
  if [[ -f "$ROOT/oracle/.env" ]]; then
    for key in SOLANA_RPC_URL SOLANA_WALLET_PUBKEY SOLANA_TOKEN_MINT; do
      val="$(grep -E "^${key}=" "$ROOT/oracle/.env" | head -1 | cut -d= -f2- || true)"
      if [[ -n "$val" && "$val" != DemoWallet* ]]; then
        sed -i "s|^${key}=.*|${key}=${val}|" "$STAGING_ENV"
      fi
    done
  fi

  chmod 600 "$STAGING_ENV"
  echo "OK — deploy/staging/.env listo para perfil local"
  echo "Luego: ./scripts/staging-local.sh up"
}

cmd_up() {
  if [[ ! -f "$STAGING_ENV" ]]; then
    cmd_init
  fi

  if [[ "$USE_DOCKER" == true ]]; then
    if ! command -v docker >/dev/null 2>&1; then
      echo "ERROR: Docker no instalado. Usá modo native: ./scripts/staging-local.sh up" >&2
      echo "  Instalar: sudo apt install docker.io docker-compose-v2" >&2
      exit 1
    fi
    "$ROOT/scripts/deploy-staging.sh" --profile local
    return
  fi

  echo "=== staging-local UP (native) ==="
  "$ROOT/scripts/check-staging-env.sh" --local
  "$ROOT/scripts/sync-staging-env-to-local.sh"

  if ! pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
    echo "ERROR: PostgreSQL no corre en 127.0.0.1:5432" >&2
    exit 1
  fi

  (cd "$ROOT/oracle" && ./scripts/setup-db.sh oracle postgres 2>/dev/null || true)

  echo "Arrancando servicios..."
  start_native antifraud antifraud 8082
  start_native binance-sim binance-sim 8083
  start_native oracle oracle 8081
  start_native gateway crates/api-gateway 8080

  echo
  echo "Stack native listo:"
  echo "  Gateway:  http://127.0.0.1:8080"
  echo "  Frontend: cd frontend && pnpm dev  → http://127.0.0.1:5173"
  echo "  Logs:     $LOG_DIR/"
  echo
  echo "Smoke: ./scripts/staging-local.sh smoke"
}

cmd_smoke() {
  local rail="${1:-all}"
  local base key
  base="$(grep -E '^PUBLIC_BASE_URL=' "$STAGING_ENV" | head -1 | cut -d= -f2- | tr -d '"')"
  key="$(grep -E '^GATEWAY_TEST_API_KEY=' "$STAGING_ENV" | head -1 | cut -d= -f2- | tr -d '"')"
  base="${base/http:\/\/localhost/http://127.0.0.1}"
  export STAGING_BASE_URL="$base"
  export GATEWAY_TEST_API_KEY="$key"
  "$ROOT/scripts/smoke-staging.sh" --rail "$rail"
}

cmd_down() {
  echo "=== staging-local DOWN ==="
  if command -v docker >/dev/null 2>&1 && [[ -f "$ROOT/deploy/staging/docker-compose.yml" ]]; then
    (cd "$ROOT/deploy/staging" && docker compose --env-file .env --profile local down 2>/dev/null) || true
  fi
  stop_native
  echo "OK"
}

cmd_status() {
  echo "=== staging-local STATUS ==="
  for pair in antifraud:8082 binance-sim:8083 oracle:8081 gateway:8080; do
    name="${pair%%:*}"
    port="${pair##*:}"
    if health "http://127.0.0.1:${port}/health"; then
      printf '  ✓ %s :%s\n' "$name" "$port"
    else
      printf '  ✗ %s :%s\n' "$name" "$port"
    fi
  done
}

case "$CMD" in
  init)   cmd_init ;;
  up)     cmd_up ;;
  smoke)
    cmd_smoke "${1:-all}"
    ;;
  down)   cmd_down ;;
  status) cmd_status ;;
  -h|--help|"")
    sed -n '1,12p' "$0"
    ;;
  *)
    echo "Comando desconocido: $CMD" >&2
    exit 2
    ;;
esac
