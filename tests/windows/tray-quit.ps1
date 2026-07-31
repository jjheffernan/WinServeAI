# Tray quit / tray-owned CLI attach proof (P1) — Windows operator gate.
# Prerequisites: built winserve-tray.exe + winserve.exe, pin binary optional for
# attach-only checks; Ready quit needs a model or fake is out of scope here.
#
# Usage (repo root):
#   .\tests\windows\tray-quit.ps1 -CheckOnly
#   .\tests\windows\tray-quit.ps1 -StartTray
#
# Pass criteria when -StartTray:
# 1. Tray acquires lock; `winserve status` attaches (not Stopped-from-no-owner).
# 2. Second winserve-tray start fails (already running).
# 3. Tray Quit (menu) leaves no llama-server.exe; lock released.

[CmdletBinding()]
param(
    [switch]$CheckOnly,
    [switch]$StartTray,
    [int]$SettleMs = 1500
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $Root

function Find-Bin([string]$Name) {
    foreach ($cfg in @("release", "debug")) {
        $p = Join-Path $Root "target\$cfg\$Name"
        if (Test-Path $p) { return $p }
    }
    return $null
}

$tray = Find-Bin "winserve-tray.exe"
$cli = Find-Bin "winserve.exe"
if (-not $tray) { throw "winserve-tray.exe not found — cargo build -p winserve-tray" }
if (-not $cli) { throw "winserve.exe not found — cargo build -p winserve" }

Write-Host @"
Tray ownership checklist
1. Start tray → lock held; .\winserve.exe status attaches
2. Second tray instance refused
3. Tray menu Quit → ServerManager::stop(); no orphan llama-server
4. Window close hides to tray (does not stop); Quit stops
"@

if ($CheckOnly) {
    Write-Host "OK: binaries present (`n  tray=$tray`n  cli=$cli)"
    exit 0
}

if (-not $StartTray) {
    Write-Host "OK: pass -StartTray to run attach/singleton smoke (manual Quit still required for orphan proof)"
    exit 0
}

$proc = Start-Process -FilePath $tray -PassThru -WindowStyle Hidden
Start-Sleep -Milliseconds $SettleMs

$status = & $cli status 2>&1 | Out-String
Write-Host "CLI status against tray owner:`n$status"
if ($status -match "already running|cannot acquire") {
    Write-Error "CLI could not attach unexpectedly"
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$second = Start-Process -FilePath $tray -PassThru -WindowStyle Hidden -Wait -RedirectStandardError (Join-Path $env:TEMP "winserve-tray-second.err")
# Second instance should exit non-zero / panic on lock — process should not stay up.
Start-Sleep -Milliseconds 500
$secondAlive = Get-Process -Id $second.Id -ErrorAction SilentlyContinue
if ($secondAlive) {
    Stop-Process -Id $second.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Write-Error "second tray instance stayed alive (singleton broken)"
    exit 1
}

Write-Host "OK: tray owner up; second instance did not stay resident."
Write-Host "Manual: use tray Quit, then confirm Get-Process llama-server fails."
Write-Host "Leaving tray pid=$($proc.Id) running for manual Quit proof."
exit 0
