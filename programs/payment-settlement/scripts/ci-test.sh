#!/usr/bin/env bash
# Gate 3.8 — reproduce CI localmente: build SBF + anchor test (validador local).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: '$1' no encontrado en PATH" >&2
    exit 1
  fi
}

require_cmd solana
require_cmd anchor
require_cmd npm

if [[ ! -f "${HOME}/.config/solana/id.json" ]]; then
  echo "Creando wallet efímera en ~/.config/solana/id.json"
  mkdir -p "${HOME}/.config/solana"
  solana-keygen new --no-bip39-passphrase -s -o "${HOME}/.config/solana/id.json"
fi

echo "==> Solana: $(solana --version)"
echo "==> Anchor: $(anchor --version)"
echo "==> Platform-tools (build): v1.52 (ver npm run build)"

npm ci
npm run lint:docs
npm test

echo "==> Gate 3.8 OK — anchor test verde"
echo "==> Gate 3.10 OK — documentación @notice/@param/@return"
