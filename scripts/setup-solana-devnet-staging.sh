#!/usr/bin/env bash
# Comprueba wallet SPL en devnet para staging (Fase 7.8).
#
# Uso:
#   ./scripts/setup-solana-devnet-staging.sh
#   ./scripts/setup-solana-devnet-staging.sh --wallet <pubkey> --mint <mint>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT/deploy/staging/.env"
WALLET=""
MINT=""
RPC="https://api.devnet.solana.com"

get_env() {
  local key="$1" file="${2:-$ENV_FILE}"
  grep -E "^${key}=" "$file" 2>/dev/null | head -1 | cut -d= -f2- \
    | sed 's/^["'\''"]//;s/["'\''"]$//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --wallet) WALLET="${2:?}"; shift 2 ;;
    --mint) MINT="${2:?}"; shift 2 ;;
    --rpc) RPC="${2:?}"; shift 2 ;;
    -h|--help)
      sed -n '1,6p' "$0"
      exit 0
      ;;
    *) echo "Argumento desconocido: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$WALLET" && -f "$ENV_FILE" ]]; then
  WALLET="$(get_env SOLANA_WALLET_PUBKEY)"
  MINT="$(get_env SOLANA_TOKEN_MINT)"
  RPC="$(get_env SOLANA_RPC_URL)"
  RPC="${RPC:-https://api.devnet.solana.com}"
fi

if [[ -z "$WALLET" || -z "$MINT" ]]; then
  echo "ERROR: definí --wallet y --mint, o completá deploy/staging/.env" >&2
  exit 1
fi

echo "=== setup-solana-devnet-staging ==="
echo "RPC:    ${RPC}"
echo "Wallet: ${WALLET}"
echo "Mint:   ${MINT}"
echo

health="$(curl -sf --max-time 10 -X POST "$RPC" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}')"
if echo "$health" | grep -q '"result"'; then
  echo "  ✓ RPC devnet OK"
else
  echo "  ✗ RPC devnet no responde: ${health}" >&2
  exit 1
fi

payload="$(printf '{"jsonrpc":"2.0","id":1,"method":"getTokenAccountsByOwner","params":["%s",{"mint":"%s"},{"encoding":"jsonParsed"}]}' \
  "$WALLET" "$MINT")"

accounts="$(curl -sf --max-time 15 -X POST "$RPC" \
  -H 'Content-Type: application/json' \
  -d "$payload")"

if echo "$accounts" | grep -q '"uiAmount"'; then
  amount="$(echo "$accounts" | grep -o '"uiAmount":[0-9.]*' | head -1 | cut -d: -f2)"
  echo "  ✓ Saldo SPL detectado: ${amount} tokens"
  if [[ "${amount:-0}" == "0" || -z "$amount" ]]; then
    echo "  ⚠ Saldo cero — acreditá tokens antes del smoke solana_wallet"
    exit 1
  fi
else
  echo "  ✗ Sin cuenta SPL para wallet/mint en devnet" >&2
  echo
  echo "Pasos sugeridos:"
  echo "  solana config set --url devnet"
  echo "  spl-token create-account ${MINT}   # o transfer desde otra wallet"
  echo "  Actualizá SOLANA_WALLET_PUBKEY y SOLANA_TOKEN_MINT en deploy/staging/.env"
  exit 1
fi

PROGRAM_ID=""
if [[ -f "$ENV_FILE" ]]; then
  PROGRAM_ID="$(get_env PAYMENT_SETTLEMENT_PROGRAM_ID)"
fi
PROGRAM_ID="${PROGRAM_ID:-4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B}"

prog_payload="$(printf '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["%s",{"encoding":"base64"}]}' "$PROGRAM_ID")"
prog="$(curl -sf --max-time 10 -X POST "$RPC" -H 'Content-Type: application/json' -d "$prog_payload")"
if echo "$prog" | grep -q '"value":{'; then
  echo "  ✓ Programa payment-settlement desplegado (${PROGRAM_ID})"
else
  echo "  ⚠ Programa ${PROGRAM_ID} no encontrado en devnet"
fi

echo
echo "Devnet listo para smoke: ./scripts/smoke-staging.sh --rail solana"
exit 0
