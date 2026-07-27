#!/usr/bin/env bash
# Cambia el stack staging local a Solana devnet (Fase 7.8) y valida.
#
# Uso:
#   ./scripts/staging-devnet.sh           # sync + reinicia oracle + smoke 3 rieles
#   ./scripts/staging-devnet.sh --check   # solo verifica RPC, programa y saldo SPL
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGING_ENV="$ROOT/deploy/staging/.env"
CHECK_ONLY=false

[[ "${1:-}" == "--check" ]] && CHECK_ONLY=true

if [[ ! -f "$STAGING_ENV" ]]; then
  echo "ERROR: ejecutá primero ./scripts/staging-local.sh init" >&2
  exit 1
fi

grep -q '^SOLANA_RPC_URL=https://api.devnet.solana.com' "$STAGING_ENV" || {
  echo "ERROR: deploy/staging/.env no apunta a devnet (SOLANA_RPC_URL)" >&2
  echo "Editá SOLANA_WALLET_PUBKEY y SOLANA_TOKEN_MINT con wallet SPL en devnet." >&2
  exit 1
}

echo "=== staging-devnet — validación contra devnet ==="
"$ROOT/scripts/setup-solana-devnet-staging.sh"

if [[ "$CHECK_ONLY" == true ]]; then
  exit 0
fi

# Stack native debe estar arriba
if ! curl -sf --max-time 2 http://127.0.0.1:8080/health >/dev/null; then
  echo "Levantando stack native..."
  "$ROOT/scripts/staging-local.sh" up
fi

"$ROOT/scripts/sync-staging-env-to-local.sh"

echo "Reiniciando Oracle con config devnet..."
fuser -k 8081/tcp 2>/dev/null || true
sleep 2
mkdir -p /tmp/pasarela-staging/pids /tmp/pasarela-staging/logs
(cd "$ROOT/oracle" && nohup cargo run -q > /tmp/pasarela-staging/logs/oracle.log 2>&1 & echo $! > /tmp/pasarela-staging/pids/oracle.pid)
for _ in $(seq 1 30); do
  curl -sf http://127.0.0.1:8081/health >/dev/null && break
  sleep 2
done

echo
"$ROOT/scripts/staging-local.sh" smoke
echo
echo "Validación devnet OK — Oracle consulta https://api.devnet.solana.com"
