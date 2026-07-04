# Windows OpenAI smoke for WinServeAI (A2).
# Polls GET /v1/models until 200, optionally POSTs /v1/chat/completions.
#
# Requires: bin\llama-server.exe from llama.cpp pin b9866 (see bin\README.md),
#           real GGUF at model.path, Windows host. Default timeout 120s.
# Stop: Ctrl+C on foreground `winserve start` (not `winserve stop`). Do not use
#       scripts\stop.ps1 as the A2 pass path (global force-kill).
#
# Exit codes:
#   0  ready (and optional chat ok)
#   1  preconditions failed (binary, config, model.path, port busy with -Start)
#   2  readiness timeout or non-200 after wait
#   3  chat completion failed (only when chat is attempted)
#   4  failed to start winserve (when -Start is used)
#
# Usage (repo root or any cwd):
#   .\scripts\smoke-openai.ps1              # expect operator already started winserve
#   .\scripts\smoke-openai.ps1 -Start       # start winserve in background, then poll
#   .\scripts\smoke-openai.ps1 -Start -SkipChat
#   .\scripts\smoke-openai.ps1 -TimeoutSec 180

[CmdletBinding()]
param(
    [switch]$Start,
    [switch]$SkipChat,
    [int]$TimeoutSec = 120,
    [int]$PollMs = 500,
    [string]$ConfigPath = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

# Set when -Start launches winserve; stopped on failure, left running on success.
$script:StartedProc = $null

function Stop-StartedWinserve {
    $p = $script:StartedProc
    if (-not $p) { return }
    try {
        if (-not $p.HasExited) {
            Write-Host "Stopping background winserve pid=$($p.Id) (Job Object should reap llama-server)"
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 500
        }
    } catch { }
    $script:StartedProc = $null
}

function Write-Fail([int]$Code, [string]$Message) {
    Stop-StartedWinserve
    Write-Host "FAIL ($Code): $Message" -ForegroundColor Red
    exit $Code
}

function Write-Ok([string]$Message) {
    Write-Host "OK: $Message" -ForegroundColor Green
}

function Resolve-ConfigPath {
    if ($ConfigPath) { return (Resolve-Path -LiteralPath $ConfigPath).Path }
    if ($env:WINSERVE_CONFIG) { return $env:WINSERVE_CONFIG }
    return (Join-Path $Root "config\default.yaml")
}

function Read-YamlScalar([string]$Text, [string]$Key) {
    # Minimal YAML scalar reader for top-level or one-indent keys (host/port/path).
    $pattern = "(?m)^\s*$([regex]::Escape($Key))\s*:\s*(.+?)\s*$"
    $m = [regex]::Match($Text, $pattern)
    if (-not $m.Success) { return $null }
    $v = $m.Groups[1].Value.Trim().Trim('"').Trim("'")
    return $v
}

function Find-WinserveExe {
    $release = Join-Path $Root "target\release\winserve.exe"
    $debug = Join-Path $Root "target\debug\winserve.exe"
    if (Test-Path -LiteralPath $release) { return $release }
    if (Test-Path -LiteralPath $debug) { return $debug }
    return $null
}

function Test-TcpOpen([string]$HostName, [int]$Port) {
    try {
        $client = New-Object System.Net.Sockets.TcpClient
        $iar = $client.BeginConnect($HostName, $Port, $null, $null)
        $ok = $iar.AsyncWaitHandle.WaitOne(500)
        if (-not $ok) {
            $client.Close()
            return $false
        }
        $client.EndConnect($iar)
        $client.Close()
        return $true
    } catch {
        return $false
    }
}

# --- Preconditions ---

$cfg = Resolve-ConfigPath
if (-not (Test-Path -LiteralPath $cfg)) {
    Write-Fail 1 "missing config at $cfg (set WINSERVE_CONFIG or pass -ConfigPath)"
}

$llama = Join-Path $Root "bin\llama-server.exe"
if (-not (Test-Path -LiteralPath $llama)) {
    Write-Fail 1 "missing bin\llama-server.exe (pin b9866 — see bin\README.md)"
}

$yaml = Get-Content -LiteralPath $cfg -Raw
$modelPath = Read-YamlScalar $yaml "path"
# Prefer model.path under model: block — first "path:" after "model:" is fine for default.yaml
if (-not $modelPath) {
    Write-Fail 1 "config has no model.path (edit $cfg)"
}
# default.yaml uses model.path as the only path: key under model:
# Re-read: the first path: in file is model.path in our schema.
$hostName = Read-YamlScalar $yaml "host"
$portStr = Read-YamlScalar $yaml "port"
if (-not $hostName) { $hostName = "127.0.0.1" }
if (-not $portStr) { $portStr = "8080" }
$port = [int]$portStr

if (-not (Test-Path -LiteralPath $modelPath)) {
    Write-Fail 1 "model.path does not exist: $modelPath (set a real .gguf in $cfg)"
}

$base = "http://${hostName}:${port}"
$modelsUrl = "$base/v1/models"
Write-Host "Smoke target: $modelsUrl"
Write-Host "Config: $cfg"
Write-Host "Binary: $llama"
Write-Host "Model:  $modelPath"

# --- Optional start ---

$proc = $null
if ($Start) {
    if (Test-TcpOpen $hostName $port) {
        Write-Fail 1 "port ${hostName}:${port} already in use (stop the other listener; do not smoke against a foreign llama-server)"
    }
    $winserve = Find-WinserveExe
    if (-not $winserve) {
        Write-Fail 4 "winserve.exe not found under target\release or target\debug (cargo build -p winserve)"
    }
    $env:WINSERVE_CONFIG = $cfg
    Write-Host "Starting: $winserve start (pin b9866 required in bin\ — see bin\README.md)"
    try {
        $proc = Start-Process -FilePath $winserve -ArgumentList "start" `
            -WorkingDirectory $Root -PassThru -WindowStyle Hidden
    } catch {
        Write-Fail 4 "failed to start winserve: $_"
    }
    if (-not $proc -or $proc.HasExited) {
        $code = if ($proc) { $proc.ExitCode } else { "n/a" }
        Write-Fail 4 "winserve exited immediately (code=$code); check logs\ and model.path"
    }
    $script:StartedProc = $proc
    Write-Host "winserve pid=$($proc.Id) (background)"
} else {
    Write-Host "Assuming winserve is already running (omit -Start, or start with: cargo run -p winserve -- start)"
}

# --- Poll readiness ---

$deadline = [datetime]::UtcNow.AddSeconds($TimeoutSec)
$lastStatus = "none"
$readyBody = $null

while ([datetime]::UtcNow -lt $deadline) {
    try {
        $resp = Invoke-WebRequest -Uri $modelsUrl -Method GET -UseBasicParsing -TimeoutSec 5
        $lastStatus = [int]$resp.StatusCode
        if ($resp.StatusCode -eq 200) {
            $readyBody = $resp.Content
            break
        }
    } catch {
        $ex = $_.Exception
        if ($ex.Response -and $ex.Response.StatusCode) {
            $lastStatus = [int]$ex.Response.StatusCode
        } else {
            $lastStatus = "connect-error"
        }
    }

    if ($proc -and $proc.HasExited) {
        Write-Fail 2 "winserve exited during wait (code=$($proc.ExitCode)); last probe=$lastStatus"
    }
    Start-Sleep -Milliseconds $PollMs
}

if (-not $readyBody) {
    Write-Fail 2 "GET $modelsUrl not 200 within ${TimeoutSec}s (last=$lastStatus)"
}

Write-Ok "GET /v1/models -> 200"
Write-Host $readyBody

# --- Optional chat ---

if (-not $SkipChat) {
    $modelId = "local"
    try {
        $parsed = $readyBody | ConvertFrom-Json
        if ($parsed.data -and $parsed.data.Count -gt 0 -and $parsed.data[0].id) {
            $modelId = [string]$parsed.data[0].id
        } else {
            Write-Host "WARN: models list empty; skipping chat completion"
            Write-Ok "smoke complete (models only)"
            if ($script:StartedProc -and -not $script:StartedProc.HasExited) {
                Write-Host "NOTE: background winserve still running (pid=$($script:StartedProc.Id)); Stop-Process -Id $($script:StartedProc.Id)"
            }
            exit 0
        }
    } catch {
        Write-Host "WARN: could not parse models JSON; using model=$modelId"
    }

    $chatUrl = "$base/v1/chat/completions"
    $body = @{
        model      = $modelId
        messages   = @(@{ role = "user"; content = "Say hi in one word." })
        max_tokens = 16
    } | ConvertTo-Json -Depth 5

    try {
        $chatResp = Invoke-WebRequest -Uri $chatUrl -Method POST -Body $body `
            -ContentType "application/json" -UseBasicParsing -TimeoutSec 120
        if ($chatResp.StatusCode -ne 200) {
            Write-Fail 3 "POST /v1/chat/completions -> $($chatResp.StatusCode)"
        }
        Write-Ok "POST /v1/chat/completions -> 200"
        Write-Host $chatResp.Content
    } catch {
        Write-Fail 3 "POST /v1/chat/completions failed: $_"
    }
}

Write-Ok "smoke complete"
if ($script:StartedProc -and -not $script:StartedProc.HasExited) {
    Write-Host "NOTE: background winserve still running (pid=$($script:StartedProc.Id))."
    Write-Host "  Cooperative stop is Ctrl+C on a foreground start (Option A in docs/specs/A2-smoke.md)."
    Write-Host "  For this -Start process: Stop-Process -Id $($script:StartedProc.Id)  # Job Object reaps llama-server"
}
exit 0
