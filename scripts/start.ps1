# Start WinServeAI (run from repo or install root)
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
$env:WINSERVE_CONFIG = Join-Path $Root "config\default.yaml"
& (Join-Path $Root "target\release\winserve.exe") start
if (-not $?) {
    & (Join-Path $Root "target\debug\winserve.exe") start
}
