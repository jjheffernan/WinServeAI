# Orphan-free quit proof (P0) — Windows only.
# Starts winserve (or expects Ready), force-kills the manager PID, asserts no
# leftover llama-server.exe. Job Object KILL_ON_JOB_CLOSE is the backstop.
#
# Usage:
#   .\tests\windows\orphan-quit.ps1 -Start
#   .\tests\windows\orphan-quit.ps1 -ManagerPid 1234

[CmdletBinding()]
param(
    [switch]$Start,
    [int]$ManagerPid = 0,
    [string]$ConfigPath = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $Root

function Get-LlamaPids {
    @(Get-Process -Name "llama-server" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
}

if ($Start) {
    $winserve = Join-Path $Root "target\release\winserve.exe"
    if (-not (Test-Path $winserve)) { $winserve = Join-Path $Root "target\debug\winserve.exe" }
    if (-not (Test-Path $winserve)) { throw "winserve.exe not found — build -p winserve first" }
    $args = @("start")
    if ($ConfigPath) { $args += @("--config", $ConfigPath) }
    $p = Start-Process -FilePath $winserve -ArgumentList $args -PassThru -WindowStyle Hidden
    $ManagerPid = $p.Id
    Start-Sleep -Seconds 3
}

if ($ManagerPid -le 0) { throw "pass -Start or -ManagerPid" }

$before = Get-LlamaPids
Write-Host "ManagerPid=$ManagerPid llama-before=$($before -join ',')"

Stop-Process -Id $ManagerPid -Force -ErrorAction Stop
Start-Sleep -Milliseconds 800

$after = Get-LlamaPids
if ($after.Count -gt 0) {
    Write-Error "orphan llama-server still running: $($after -join ',')"
    exit 1
}
Write-Host "OK: no orphan llama-server after manager kill"
exit 0
