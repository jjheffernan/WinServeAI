# Stop processes and clear log files (keeps config and models).
$Root = Split-Path -Parent $PSScriptRoot
& "$PSScriptRoot\stop.ps1"
$LogDir = Join-Path $Root "logs"
if (Test-Path $LogDir) {
    Get-ChildItem $LogDir -File | Remove-Item -Force
    Write-Host "Cleared $LogDir"
}
