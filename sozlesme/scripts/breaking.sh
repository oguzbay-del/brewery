#!/usr/bin/env bash
# Fails when sozlesme/openapi/openapi.json is not backward compatible with the one on the base
# branch (ADR-0008 rule 8.2). Compatible changes (new endpoints, new response fields, new enum
# values) pass; removals and type changes need `/v2` or an entry in
# sozlesme/openapi/oasdiff-ignore.txt (pre-release only).
#
# Usage: bash sozlesme/scripts/breaking.sh [base-ref]   (default: origin/main)
# oasdiff is pinned: go install github.com/oasdiff/oasdiff@v1.37.0
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

BASE_REF="${1:-origin/main}"
SPEC="sozlesme/openapi/openapi.json"
IGNORE="sozlesme/openapi/oasdiff-ignore.txt"
OASDIFF="${OASDIFF:-oasdiff}"

if ! command -v "$OASDIFF" >/dev/null 2>&1; then
  if [ -n "${CI:-}" ]; then
    echo "error: oasdiff is not installed (go install github.com/oasdiff/oasdiff@v1.37.0)" >&2
    exit 1
  fi
  echo "skip: oasdiff not installed; compatibility with $BASE_REF not checked."
  exit 0
fi

BASE_FILE="$(mktemp)"
trap 'rm -f "$BASE_FILE"' EXIT
if ! git show "$BASE_REF:$SPEC" >"$BASE_FILE" 2>/dev/null; then
  echo "skip: $BASE_REF has no $SPEC (first version); nothing to compare."
  exit 0
fi

ARGS=(breaking "$BASE_FILE" "$SPEC" --fail-on ERR)
if [ -f "$IGNORE" ]; then
  ARGS+=(--err-ignore "$IGNORE")
fi
"$OASDIFF" "${ARGS[@]}"
echo "ok: $SPEC is backward compatible with $BASE_REF."
