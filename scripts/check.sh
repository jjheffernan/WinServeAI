#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo check"
cargo check --workspace

echo "==> cargo test"
cargo test --workspace

echo "OK"
