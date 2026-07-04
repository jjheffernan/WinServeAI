#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo check"
cargo check -p winserve

echo "==> cargo test"
cargo test -p winserve

echo "OK"
