# Install → Ready checklist (P0). Manual/operator gate — ISCC + clean machine.
# Does not auto-install; prints the acceptance steps and exits 0 when preconditions
# for starting the checklist are present (ISCC optional).
#
# Usage:
#   .\tests\windows\install-ready.ps1
#   .\tests\windows\install-ready.ps1 -CheckOnly

[CmdletBinding()]
param([switch]$CheckOnly)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$iss = Join-Path $Root "installer\inno\WinServeAI.iss"
$stage = Join-Path $Root "scripts\stage-release.ps1"

Write-Host @"
Install → Ready checklist
1. Stage:  .\scripts\stage-release.ps1 -RequireLlama
2. Compile: ISCC.exe installer\inno\WinServeAI.iss
3. Clean VM install (Program Files\WinServeAI)
4. FIRST_RUN.txt / Settings → Browse .gguf
5. Start tray → Ready; GET /v1/models
6. Uninstall → firewall rule for llama-server.exe gone
7. Shortcuts never point at llama-server.exe
"@

if (-not (Test-Path $iss)) { throw "missing $iss" }
if (-not (Test-Path $stage)) { throw "missing $stage" }

$iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue
if (-not $iscc) {
    Write-Warning "ISCC.exe not on PATH — compile step is operator/CI only"
} else {
    Write-Host "ISCC: $($iscc.Source)"
}

if ($CheckOnly) {
    Write-Host "OK: checklist preconditions present"
    exit 0
}

Write-Host "OK: run the steps above on a clean Windows machine; this script does not install."
exit 0
