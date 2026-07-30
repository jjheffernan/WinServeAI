# Stop WinServeAI for local dev.
#
# Prefers graceful IPC stop when a resident lockfile exists
# (%LOCALAPPDATA%\WinServeAI\manager.lock). Force-kill remains the fallback
# (and is available via -Force).

param(
    [switch]$Force
)

$ErrorActionPreference = "Continue"
$Root = Split-Path -Parent $PSScriptRoot
$LockPath = Join-Path $env:LOCALAPPDATA "WinServeAI\manager.lock"

function Find-Winserve {
    $candidates = @(
        (Join-Path $Root "target\release\winserve.exe"),
        (Join-Path $Root "target\debug\winserve.exe"),
        (Join-Path $Root "winserve.exe")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    $cmd = Get-Command winserve -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

function Test-ResidentLockAlive {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return $false }
    try {
        $info = Get-Content -Raw -Path $Path | ConvertFrom-Json
        if (-not $info.pid) { return $false }
        $proc = Get-Process -Id ([int]$info.pid) -ErrorAction SilentlyContinue
        return $null -ne $proc
    }
    catch {
        return $false
    }
}

function Stop-Force {
    Get-Process -Name "winserve", "llama-server" -ErrorAction SilentlyContinue |
        Stop-Process -Force
    Write-Host "Force-stopped winserve / llama-server (if running)."
}

if ($Force) {
    Write-Host "Force requested; skipping IPC."
    Stop-Force
    exit 0
}

if (Test-ResidentLockAlive -Path $LockPath) {
    $exe = Find-Winserve
    if ($exe) {
        Write-Host "Resident lock found; graceful stop via IPC ($exe stop)…"
        & $exe stop
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Stopped via resident manager IPC."
            exit 0
        }
        Write-Host "IPC stop failed (exit $LASTEXITCODE); falling back to force."
    }
    else {
        Write-Host "Lockfile present but winserve.exe not found; falling back to force."
    }
}
else {
    Write-Host "No live resident lockfile; force-stopping by process name."
}

Stop-Force
