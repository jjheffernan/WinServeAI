#!/usr/bin/env bash
# Host-safe ops script check (macOS/Linux/CI). Does NOT run Windows inference
# or require PowerShell execution of start/stop/smoke against a live server.
#
# Verifies:
#   - required operator scripts exist
#   - key contracts appear in script text (A2 wrapper, lock-aware stop, pin note)
#   - optional: pwsh/powershell parse syntax when available
#
# Exit: 0 on success, non-zero on first failure.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

ok() {
  echo "OK: $*"
}

require_file() {
  local f="$1"
  [[ -f "$ROOT/$f" ]] || fail "missing $f"
  ok "present $f"
}

require_grep() {
  local f="$1"
  local pat="$2"
  local label="$3"
  grep -qE "$pat" "$ROOT/$f" || fail "$f missing expected pattern ($label): $pat"
  ok "$f has $label"
}

require_file "scripts/start.ps1"
require_file "scripts/stop.ps1"
require_file "scripts/reset.ps1"
require_file "scripts/smoke-openai.ps1"
require_file "scripts/smoke-check.sh"
require_file "tests/windows/a2-smoke.ps1"
require_file "tests/windows/orphan-quit.ps1"
require_file "tests/windows/install-ready.ps1"
require_file "tests/windows/tray-quit.ps1"

require_grep "scripts/stop.ps1" "manager\\.lock|winserve stop" "lock-aware stop"
require_grep "scripts/smoke-openai.ps1" "b9866|/v1/models" "A2 pin or models poll"
require_grep "tests/windows/a2-smoke.ps1" "smoke-openai\\.ps1" "A2 delegates to smoke-openai"
require_grep "tests/windows/orphan-quit.ps1" "llama-server|Get-CimInstance|Get-Process" "orphan process check"
require_grep "tests/windows/tray-quit.ps1" "Quit|llama-server|manager\\.lock" "tray quit contract"

# Optional syntax parse — skip quietly when neither pwsh nor Windows PowerShell exists.
PS_BIN=""
if command -v pwsh >/dev/null 2>&1; then
  PS_BIN="pwsh"
elif command -v powershell >/dev/null 2>&1; then
  PS_BIN="powershell"
fi

if [[ -n "$PS_BIN" ]]; then
  for f in \
    scripts/start.ps1 \
    scripts/stop.ps1 \
    scripts/reset.ps1 \
    scripts/smoke-openai.ps1 \
    tests/windows/a2-smoke.ps1 \
    tests/windows/orphan-quit.ps1 \
    tests/windows/install-ready.ps1 \
    tests/windows/tray-quit.ps1
  do
    "$PS_BIN" -NoProfile -Command "& { \$null = [System.Management.Automation.Language.Parser]::ParseFile('$ROOT/$f', [ref]\$null, [ref]\$errs); if (\$errs) { \$errs | ForEach-Object { \$_.ToString() }; exit 1 } }" \
      || fail "PowerShell parse failed: $f"
    ok "parsed $f"
  done
else
  echo "NOTE: no pwsh/powershell on PATH — skipped syntax parse (content checks still ran)"
fi

echo "host-safe-ops-check: OK"
