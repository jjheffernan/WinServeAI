#!/usr/bin/env bash
# Stage a WinServeAI {app} release layout (F0). Prefer scripts/stage-release.ps1 on Windows.
#
# Examples:
#   ./scripts/stage-release.sh
#   ./scripts/stage-release.sh --skip-build --out dist/WinServeAI
#   CONFIGURATION=debug ./scripts/stage-release.sh --skip-build

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${OUT:-$ROOT/dist/WinServeAI}"
CONFIGURATION="${CONFIGURATION:-release}"
SKIP_BUILD=0
REQUIRE_LLAMA=0
TARGET_ROOT="${CARGO_TARGET_DIR:-$ROOT/target}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --configuration) CONFIGURATION="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --require-llama) REQUIRE_LLAMA=1; shift ;;
    -h|--help)
      echo "Usage: $0 [--out DIR] [--configuration release|debug] [--skip-build] [--require-llama]"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

TARGET="$TARGET_ROOT/$CONFIGURATION"
BIN_SRC="$ROOT/bin"

echo "Root:   $ROOT"
echo "OutDir: $OUT"
echo "Config: $CONFIGURATION"

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  echo "Building winserve ($CONFIGURATION)…"
  (cd "$ROOT" && cargo build -p winserve "--$CONFIGURATION")
  echo "Building winserve-tray ($CONFIGURATION)…"
  (cd "$ROOT" && cargo build -p winserve-tray "--$CONFIGURATION") || {
    echo "WARN: winserve-tray build failed; continuing without tray binary" >&2
  }
fi

rm -rf "$OUT"
mkdir -p "$OUT/bin" "$OUT/config" "$OUT/notices" "$OUT/logs"
: >"$OUT/logs/.gitkeep"

copy_bin() {
  local name="$1"
  local src="$TARGET/$name"
  local src_exe="$TARGET/${name}.exe"
  if [[ -f "$src_exe" ]]; then
    cp -f "$src_exe" "$OUT/${name}.exe"
    echo "OK: ${name}.exe"
  elif [[ -f "$src" ]]; then
    cp -f "$src" "$OUT/$name"
    echo "OK: $name"
  else
    return 1
  fi
}

copy_bin winserve || {
  echo "error: missing winserve under $TARGET (build first or pass --skip-build after a build)" >&2
  exit 1
}
copy_bin winserve-tray || echo "WARN: winserve-tray missing under $TARGET"

[[ -f "$ROOT/config/default.yaml" ]] || {
  echo "error: missing config/default.yaml" >&2
  exit 1
}
cp -f "$ROOT/config/default.yaml" "$OUT/config/default.yaml"
echo "OK: config/default.yaml"

if [[ -d "$ROOT/notices" ]]; then
  cp -R "$ROOT/notices/." "$OUT/notices/"
  echo "OK: notices/"
else
  echo "WARN: notices/ missing in repo"
fi

if [[ -f "$BIN_SRC/llama-server.exe" || -f "$BIN_SRC/llama-server" ]]; then
  # Copy siblings (DLLs) beside the server binary; skip README/.gitkeep noise is fine.
  cp -R "$BIN_SRC"/. "$OUT/bin/" 2>/dev/null || true
  # Do not ship bin/README.md as a runtime requirement — leave it if present.
  echo "OK: bin/ (from repo bin/)"
else
  msg="bin/llama-server(.exe) not found — run scripts/fetch-llama-pin.sh|.ps1 first"
  if [[ "$REQUIRE_LLAMA" -eq 1 ]]; then
    echo "error: $msg" >&2
    exit 1
  fi
  echo "WARN: $msg"
fi

echo
echo "Staged {app} layout at: $OUT"
(cd "$OUT" && find . -type f | sort | sed 's|^\./|  |')
