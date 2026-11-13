#!/usr/bin/env bash
# Regenerates the OpenAPI document and the TypeScript types. Does not check anything.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

(cd "$ROOT/sunucu" && cargo run --quiet -p api --bin gen-openapi)

TS_DIR="$ROOT/sozlesme/istemci/typescript"
if [ ! -d "$TS_DIR/node_modules" ]; then
  (cd "$TS_DIR" && npm ci --no-audit --no-fund)
fi
(cd "$TS_DIR" && npm run --silent gen)

# Field, unit and text catalog: TypeScript and Swift constants, plus the copy Rust embeds.
python3 -I "$ROOT/sozlesme/katalog/uret.py"
