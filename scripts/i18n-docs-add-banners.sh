#!/usr/bin/env bash
# Añade banner bilingüe al inicio de cada .md original (si aún no lo tiene).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

add_banner() {
  local file="$1"
  local dir base name
  dir="$(dirname "$file")"
  base="$(basename "$file" .md)"
  [[ "$base" == *-es ]] || [[ "$base" == *-en ]] && return 0
  grep -q 'Documentation / Documentación' "$file" 2>/dev/null && return 0

  name="$base"
  rel_es="${name}-es.md"
  rel_en="${name}-en.md"

  local tmp
  tmp="$(mktemp)"
  {
    echo "> **Documentation / Documentación:** [Español (es)](${rel_es}) · [English (en)](${rel_en})"
    echo ">"
    cat "$file"
  } > "$tmp"
  mv "$tmp" "$file"
}

export -f add_banner
export ROOT

while IFS= read -r -d '' file; do
  add_banner "$file"
done < <(
  find "$ROOT" \
    \( -path '*/node_modules/*' -o -path '*/target/*' -o -path '*/.git/*' \) -prune \
    -o -type f -name '*.md' ! -name '*-es.md' ! -name '*-en.md' -print0
)

echo "OK — banners añadidos a archivos originales"
