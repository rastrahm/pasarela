#!/usr/bin/env bash
# Verifica que payment-settlement esté desplegado en devnet (paso 3.9).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOY_JSON="$ROOT/deploy/devnet.json"
PROGRAM_ID="$(jq -r '.programId' "$DEPLOY_JSON")"
RPC_URL="$(jq -r '.rpcUrl' "$DEPLOY_JSON")"

echo "Verificando programa $PROGRAM_ID en $RPC_URL ..."
solana program show "$PROGRAM_ID" --url "$RPC_URL"
echo "OK — programa activo en devnet."
