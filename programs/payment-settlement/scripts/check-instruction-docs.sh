#!/usr/bin/env bash
# Gate 3.10 — cada instrucción pública debe tener @notice, @param y @return.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$ROOT/programs/payment-settlement/src/lib.rs"

INSTRUCTIONS=(
  initialize
  initialize_settlement
  set_settlement_totals
  initialize_settlement_undersized
  process_payment
)

missing=0

for fn in "${INSTRUCTIONS[@]}"; do
  line_num="$(grep -n "pub fn ${fn}" "$LIB" | head -1 | cut -d: -f1)"
  if [[ -z "$line_num" ]]; then
    echo "FAIL: no se encontró pub fn ${fn}()" >&2
    missing=$((missing + 1))
    continue
  fi

  # Doc block precede la firma de la función (hasta 20 líneas arriba).
  block="$(sed -n "$((line_num - 20)),${line_num}p" "$LIB")"

  for tag in @notice @param @return; do
    if ! grep -q "$tag" <<< "$block"; then
      echo "FAIL: ${fn}() falta ${tag}" >&2
      missing=$((missing + 1))
    fi
  done
done

if [[ "$missing" -gt 0 ]]; then
  echo "${missing} instrucción(es) sin documentación obligatoria." >&2
  exit 1
fi

echo "OK — ${#INSTRUCTIONS[@]} instrucciones documentadas (@notice/@param/@return)."
