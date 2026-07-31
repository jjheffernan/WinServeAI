# Windows A2 smoke wrapper — delegates to scripts/smoke-openai.ps1.
# Requires: Win10/11, bin\llama-server.exe (pin b9866), real .gguf at model.path.
#
# Usage (repo root):
#   .\tests\windows\a2-smoke.ps1
#   .\tests\windows\a2-smoke.ps1 -Start -SkipChat

[CmdletBinding()]
param(
    [switch]$Start,
    [switch]$SkipChat,
    [int]$TimeoutSec = 120,
    [string]$ConfigPath = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
& "$Root\scripts\smoke-openai.ps1" -Start:$Start -SkipChat:$SkipChat -TimeoutSec $TimeoutSec -ConfigPath $ConfigPath
exit $LASTEXITCODE
