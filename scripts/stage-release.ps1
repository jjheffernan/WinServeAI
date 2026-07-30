# Stage a WinServeAI {app} release layout for installer / zip payloads (F0).
#
# Produces:
#   <OutDir>\
#     winserve.exe
#     winserve-tray.exe   (if built)
#     bin\                (llama-server.exe + sibling DLLs when present)
#     config\default.yaml
#     notices\…
#     logs\.gitkeep
#
# Examples:
#   .\scripts\stage-release.ps1
#   .\scripts\stage-release.ps1 -SkipBuild -OutDir dist\WinServeAI
#   .\scripts\stage-release.ps1 -Configuration release

param(
    [string]$OutDir = "",
    [ValidateSet("release", "debug")]
    [string]$Configuration = "release",
    [switch]$SkipBuild,
    [switch]$RequireLlama
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
if (-not $OutDir) {
    $OutDir = Join-Path $Root "dist\WinServeAI"
}

$TargetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root "target" }
$TargetDir = Join-Path $TargetRoot $Configuration
$BinSrc = Join-Path $Root "bin"
$ConfigSrc = Join-Path $Root "config\default.yaml"
$NoticesSrc = Join-Path $Root "notices"

Write-Host "Root:   $Root"
Write-Host "OutDir: $OutDir"
Write-Host "Config: $Configuration"

if (-not $SkipBuild) {
    Write-Host "Building winserve ($Configuration)…"
    Push-Location $Root
    try {
        cargo build -p winserve --$Configuration
        Write-Host "Building winserve-tray ($Configuration)…"
        cargo build -p winserve-tray --$Configuration
    }
    finally {
        Pop-Location
    }
}

function Ensure-Dir([string]$Path) {
    New-Item -ItemType Directory -Force -Path $Path | Out-Null
}

if (Test-Path $OutDir) {
    Write-Host "Cleaning $OutDir"
    Remove-Item -Recurse -Force $OutDir
}
Ensure-Dir $OutDir
Ensure-Dir (Join-Path $OutDir "bin")
Ensure-Dir (Join-Path $OutDir "config")
Ensure-Dir (Join-Path $OutDir "notices")
Ensure-Dir (Join-Path $OutDir "logs")
Set-Content -Path (Join-Path $OutDir "logs\.gitkeep") -Value ""

$winserve = Join-Path $TargetDir "winserve.exe"
if (-not (Test-Path $winserve)) {
    # Non-Windows target dirs use no .exe
    $alt = Join-Path $TargetDir "winserve"
    if (Test-Path $alt) { $winserve = $alt }
}
if (-not (Test-Path $winserve)) {
    throw "missing winserve binary under $TargetDir (build first or drop -SkipBuild)"
}
Copy-Item -Force $winserve -Destination (Join-Path $OutDir (Split-Path -Leaf $winserve))
Write-Host "OK: $(Split-Path -Leaf $winserve)"

$trayCandidates = @(
    (Join-Path $TargetDir "winserve-tray.exe"),
    (Join-Path $TargetDir "winserve-tray")
)
$tray = $trayCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($tray) {
    Copy-Item -Force $tray -Destination (Join-Path $OutDir (Split-Path -Leaf $tray))
    Write-Host "OK: $(Split-Path -Leaf $tray)"
}
else {
    Write-Host "WARN: winserve-tray not found under $TargetDir (desktop shell missing from layout)"
}

if (-not (Test-Path $ConfigSrc)) {
    throw "missing $ConfigSrc"
}
Copy-Item -Force $ConfigSrc -Destination (Join-Path $OutDir "config\default.yaml")
Write-Host "OK: config\default.yaml"

if (Test-Path $NoticesSrc) {
    Copy-Item -Force -Recurse (Join-Path $NoticesSrc "*") -Destination (Join-Path $OutDir "notices")
    Write-Host "OK: notices\"
}
else {
    Write-Host "WARN: notices\ missing in repo"
}

$llamaExe = Join-Path $BinSrc "llama-server.exe"
$llamaUnix = Join-Path $BinSrc "llama-server"
if (Test-Path $llamaExe) {
    Copy-Item -Force (Join-Path $BinSrc "*") -Destination (Join-Path $OutDir "bin")
    Write-Host "OK: bin\ (from repo bin/, including llama-server.exe + DLLs)"
}
elseif (Test-Path $llamaUnix) {
    Copy-Item -Force (Join-Path $BinSrc "*") -Destination (Join-Path $OutDir "bin")
    Write-Host "OK: bin\ (from repo bin/, including llama-server)"
}
else {
    $msg = "bin\llama-server(.exe) not found — run scripts\fetch-llama-pin.ps1 first"
    if ($RequireLlama) { throw $msg }
    Write-Host "WARN: $msg"
}

Write-Host ""
Write-Host "Staged {app} layout at: $OutDir"
Get-ChildItem -Recurse $OutDir | ForEach-Object {
    $rel = $_.FullName.Substring($OutDir.Length).TrimStart("\", "/")
    if (-not $_.PSIsContainer) { Write-Host "  $rel" }
}
