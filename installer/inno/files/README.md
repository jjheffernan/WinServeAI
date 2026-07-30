# Staged `{app}` payload for Inno (F0 / F1).
#
# Populate with:
#   .\scripts\stage-release.ps1 -OutDir installer\inno\files
#   # or
#   ./scripts/stage-release.sh --out installer/inno/files
#
# Expected after staging:
#   winserve.exe
#   winserve-tray.exe
#   bin\llama-server.exe (+ DLLs)
#   config\default.yaml
#   notices\
#   logs\

Do not commit built binaries from this folder (gitignored).
