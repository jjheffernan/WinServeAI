#!/usr/bin/env bash
# Fetch pinned llama.cpp release asset into bin/ (macOS/Linux helper).
# Windows operators should prefer scripts/fetch-llama-pin.ps1.
#
# Examples:
#   ./scripts/fetch-llama-pin.sh
#   ./scripts/fetch-llama-pin.sh --variant win-cpu
#   PIN=b9866 ./scripts/fetch-llama-pin.sh --variant win-cuda-12.4

set -euo pipefail

PIN="${PIN:-b9866}"
VARIANT="win-cpu"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/bin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pin) PIN="$2"; shift 2 ;;
    --variant) VARIANT="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 [--pin b9866] [--variant win-cpu|win-cuda-12.4|win-cuda-13.3|win-vulkan|macos-arm64|ubuntu-x64]"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

case "$VARIANT" in
  win-cpu) ASSET="llama-${PIN}-bin-win-cpu-x64.zip"; EXE_NAME="llama-server.exe" ;;
  win-cuda-12.4) ASSET="llama-${PIN}-bin-win-cuda-12.4-x64.zip"; EXE_NAME="llama-server.exe" ;;
  win-cuda-13.3) ASSET="llama-${PIN}-bin-win-cuda-13.3-x64.zip"; EXE_NAME="llama-server.exe" ;;
  win-vulkan) ASSET="llama-${PIN}-bin-win-vulkan-x64.zip"; EXE_NAME="llama-server.exe" ;;
  macos-arm64) ASSET="llama-${PIN}-bin-macos-arm64.tar.gz"; EXE_NAME="llama-server" ;;
  ubuntu-x64) ASSET="llama-${PIN}-bin-ubuntu-x64.tar.gz"; EXE_NAME="llama-server" ;;
  *) echo "unsupported variant: $VARIANT" >&2; exit 2 ;;
esac

URL="https://github.com/ggml-org/llama.cpp/releases/download/${PIN}/${ASSET}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/winserve-llama.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$BIN"
echo "Pin:     $PIN"
echo "Variant: $VARIANT"
echo "Asset:   $ASSET"
echo "URL:     $URL"
echo "Dest:    $BIN"

echo "Downloading…"
curl -fsSL "$URL" -o "$WORK/$ASSET"

echo "Extracting…"
case "$ASSET" in
  *.zip)
    if command -v unzip >/dev/null 2>&1; then
      unzip -q "$WORK/$ASSET" -d "$WORK/out"
    else
      python3 - "$WORK/$ASSET" "$WORK/out" <<'PY'
import sys, zipfile, pathlib
pathlib.Path(sys.argv[2]).mkdir(parents=True, exist_ok=True)
zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])
PY
    fi
    ;;
  *.tar.gz)
    mkdir -p "$WORK/out"
    tar -xzf "$WORK/$ASSET" -C "$WORK/out"
    ;;
esac

FOUND="$(find "$WORK/out" -type f -name "$EXE_NAME" | head -n 1 || true)"
if [[ -z "$FOUND" ]]; then
  echo "error: $EXE_NAME not found in $ASSET" >&2
  exit 1
fi

SRC_DIR="$(dirname "$FOUND")"
echo "Copying from $FOUND"
# Copy sibling runtime files (DLLs / dylibs) beside the server binary.
cp -R "$SRC_DIR"/. "$BIN"/

DEST="$BIN/$EXE_NAME"
if [[ ! -f "$DEST" ]]; then
  echo "error: copy failed; missing $DEST" >&2
  exit 1
fi
chmod +x "$DEST" 2>/dev/null || true
echo "OK: $DEST"
printf '%s\n' "$PIN" > "$BIN/VERSION"
echo "OK: bin/VERSION = $PIN"
echo "Notices stub: notices/THIRD_PARTY_NOTICES.md (refresh on pin bumps)."
