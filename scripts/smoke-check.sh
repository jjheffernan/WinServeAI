#!/usr/bin/env bash
# Non-Windows / CI preflight for A2. Does NOT run inference or require a GGUF.
#
# Pin reminder: Windows operators need bin/llama-server.exe from llama.cpp b9866
# (see bin/README.md). Full OpenAI smoke is Windows-only: scripts/smoke-openai.ps1.
#
# Checks:
#   - config/default.yaml exists (or WINSERVE_CONFIG)
#   - cargo test -p winserve
#   - cargo run -p winserve -- print-cmd (argv resolves)
#   - documents expected Windows binary path (bin/llama-server.exe)
#
# Exit: 0 on success, non-zero on first failure.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

ok() {
  echo "OK: $*"
}

CFG="${WINSERVE_CONFIG:-$ROOT/config/default.yaml}"
if [[ ! -f "$CFG" ]]; then
  fail "missing config at $CFG (set WINSERVE_CONFIG or add config/default.yaml)"
fi
ok "config present: $CFG"

if ! grep -qE '^[[:space:]]*path:[[:space:]]*.+' "$CFG"; then
  fail "config has no model.path entry (edit $CFG; value may be a Windows path)"
fi
ok "model.path key present (existence of GGUF not required on this host)"

echo "==> cargo test -p winserve"
cargo test -p winserve
ok "cargo test -p winserve"

echo "==> cargo run -p winserve -- print-cmd"
PRINT_CMD_OUT="$(cargo run -q -p winserve -- print-cmd)"
echo "$PRINT_CMD_OUT"
if [[ -z "$PRINT_CMD_OUT" ]]; then
  fail "print-cmd produced no output"
fi
ok "print-cmd resolved argv"

# Binary path is Windows-primary; document only on non-Windows.
WIN_BIN="$ROOT/bin/llama-server.exe"
UNIX_BIN="$ROOT/bin/llama-server"
if [[ -f "$WIN_BIN" ]]; then
  ok "found $WIN_BIN (Windows pin — see bin/README.md, b9866)"
elif [[ -f "$UNIX_BIN" ]]; then
  ok "found $UNIX_BIN (optional non-Windows experiment binary)"
else
  echo "NOTE: no llama-server binary in bin/ (expected on Windows operators:"
  echo "      bin/llama-server.exe from llama.cpp release b9866 — see bin/README.md)"
  echo "      Full OpenAI smoke is Windows-only: scripts/smoke-openai.ps1"
  ok "binary path documented (not present on this machine)"
fi

echo
echo "A2 preflight complete (no inference)."
echo "On Windows with GGUF + bin/llama-server.exe:"
echo "  cargo run -p winserve -- start"
echo "  .\\scripts\\smoke-openai.ps1"
echo "Operator checklist: docs/specs/A2-smoke.md"
ok "smoke-check done"
