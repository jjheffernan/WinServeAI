#!/usr/bin/env bash
# Refresh notices/ from the pinned llama.cpp tag (F4).
#
# Usage:
#   ./scripts/refresh-notices.sh
#   PIN=b9866 ./scripts/refresh-notices.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIN="${PIN:-b9866}"
DIR="$ROOT/notices"
REPO="ggml-org/llama.cpp"

mkdir -p "$DIR"

fetch() {
  local path="$1"
  local out="$2"
  echo "Fetching $REPO@$PIN:$path → $out"
  gh api "repos/$REPO/contents/$path?ref=$PIN" --jq .content | base64 -d >"$out"
}

fetch LICENSE "$DIR/llama.cpp-LICENSE.txt"
fetch AUTHORS "$DIR/llama.cpp-AUTHORS.txt"

# Keep NVIDIA notice and rollup under version control; remind operator to bump pin in rollup.
if ! grep -q "$PIN" "$DIR/THIRD_PARTY_NOTICES.md"; then
  echo "WARN: notices/THIRD_PARTY_NOTICES.md does not mention pin $PIN — update the rollup table." >&2
fi

echo "OK: refreshed llama.cpp LICENSE + AUTHORS for $PIN"
ls -la "$DIR"
