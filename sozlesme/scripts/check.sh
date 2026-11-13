#!/usr/bin/env bash
# Regenerates the contract files and fails (exit 1) if they differ from the committed ones.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

bash sozlesme/scripts/gen.sh

PATHS=(sozlesme/openapi sozlesme/istemci/typescript/src sozlesme/istemci/swift sunucu/crates/common/src/catalog)
# Untracked files count as changes too (intent-to-add makes them visible to git diff).
git add -N -- "${PATHS[@]}" 2>/dev/null || true

if ! git diff --exit-code -- "${PATHS[@]}"; then
  echo "error: API contract is out of date; run 'bash sozlesme/scripts/gen.sh' and commit the result." >&2
  exit 1
fi
echo "ok: API contract is up to date."

python3 -I sozlesme/katalog/test.py
(cd sozlesme/istemci/typescript && npm run --silent typecheck)
