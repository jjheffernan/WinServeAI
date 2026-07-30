#!/usr/bin/env bash
# Host-safe stage-release layout asserts (P2). No llama binary required.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

TARGET="$TMP/target"
OUT="$TMP/out"
mkdir -p "$TARGET/debug"
# Dummy manager binary so --skip-build can stage.
printf 'stub' >"$TARGET/debug/winserve"
chmod +x "$TARGET/debug/winserve"

CARGO_TARGET_DIR="$TARGET" CONFIGURATION=debug \
  "$ROOT/scripts/stage-release.sh" --skip-build --out "$OUT"

test -f "$OUT/winserve" || test -f "$OUT/winserve.exe"
test -f "$OUT/config/default.yaml"
test -f "$OUT/FIRST_RUN.txt"
test -d "$OUT/notices"
test -f "$OUT/notices/THIRD_PARTY_NOTICES.md" || test -n "$(ls -A "$OUT/notices" 2>/dev/null || true)"
test -d "$OUT/logs"

# Pin record stages even without llama-server binary.
if [[ -f "$ROOT/bin/VERSION" ]]; then
  test -f "$OUT/bin/VERSION"
  grep -q . "$OUT/bin/VERSION"
fi

# --require-llama must fail when binary absent.
set +e
CARGO_TARGET_DIR="$TARGET" CONFIGURATION=debug \
  "$ROOT/scripts/stage-release.sh" --skip-build --require-llama --out "$TMP/out-req" >/dev/null 2>&1
rc=$?
set -e
[[ "$rc" -ne 0 ]] || { echo "expected --require-llama to fail without binary"; exit 1; }

echo "OK: stage-release asserts passed"
