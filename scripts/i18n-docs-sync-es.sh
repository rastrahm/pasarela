#!/usr/bin/env bash
# Copia cada .md del proyecto a su variante -es.md (sin tocar -en ni archivos ya sufijados).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
count=0

while IFS= read -r -d '' file; do
  base="${file%.md}"
  [[ "$base" == *-es ]] && continue
  [[ "$base" == *-en ]] && continue
  dest="${base}-es.md"
  cp "$file" "$dest"
  count=$((count + 1))
done < <(
  find "$ROOT" \
    \( -path '*/node_modules/*' -o -path '*/target/*' -o -path '*/.git/*' \) -prune \
    -o -type f -name '*.md' ! -name '*-es.md' ! -name '*-en.md' -print0
)

echo "OK — ${count} archivos copiados a *-es.md"
