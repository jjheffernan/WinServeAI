# Fetch pinned llama.cpp Windows binaries into bin/
#
# Default pin: b9866 (see bin/README.md). Does not commit binaries (gitignored).
#
# Examples:
#   .\scripts\fetch-llama-pin.ps1
#   .\scripts\fetch-llama-pin.ps1 -Variant cuda-12.4
#   .\scripts\fetch-llama-pin.ps1 -Pin b9866 -Variant vulkan

param(
    [string]$Pin = "b9866",
    [ValidateSet("cpu", "cuda-12.4", "cuda-13.3", "vulkan")]
    [string]$Variant = "cpu",
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}
$BinDir = Join-Path $RepoRoot "bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$asset = switch ($Variant) {
    "cpu" { "llama-$Pin-bin-win-cpu-x64.zip" }
    "cuda-12.4" { "llama-$Pin-bin-win-cuda-12.4-x64.zip" }
    "cuda-13.3" { "llama-$Pin-bin-win-cuda-13.3-x64.zip" }
    "vulkan" { "llama-$Pin-bin-win-vulkan-x64.zip" }
}

$url = "https://github.com/ggml-org/llama.cpp/releases/download/$Pin/$asset"
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("winserve-llama-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $work | Out-Null
$zip = Join-Path $work $asset

Write-Host "Pin:     $Pin"
Write-Host "Variant: $Variant"
Write-Host "Asset:   $asset"
Write-Host "URL:     $url"
Write-Host "Dest:    $BinDir"

try {
    Write-Host "Downloading…"
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing

    Write-Host "Extracting…"
    Expand-Archive -Path $zip -DestinationPath $work -Force

    $server = Get-ChildItem -Path $work -Recurse -Filter "llama-server.exe" |
        Select-Object -First 1
    if (-not $server) {
        throw "llama-server.exe not found inside $asset"
    }

    $srcDir = $server.Directory.FullName
    Write-Host "Copying from $($server.FullName)"
    Copy-Item -Force (Join-Path $srcDir "*") -Destination $BinDir

    $destExe = Join-Path $BinDir "llama-server.exe"
    if (-not (Test-Path $destExe)) {
        throw "copy failed; missing $destExe"
    }

    Write-Host "OK: $destExe"
    Write-Host "Keep CUDA/runtime DLLs beside llama-server.exe (already copied when present)."
    Write-Host "Notices stub: notices\THIRD_PARTY_NOTICES.md (refresh on pin bumps)."
}
finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
