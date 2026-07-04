# Stop any llama-server / winserve processes started for local dev.
Get-Process -Name "winserve","llama-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Write-Host "Stopped winserve / llama-server (if running)."
