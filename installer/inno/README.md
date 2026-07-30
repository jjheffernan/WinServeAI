# Inno Setup (F1–F3)

## Build installer

```powershell
# 1. Stage {app} payload (requires release builds + pinned llama-server)
.\scripts\stage-release.ps1 -OutDir installer\inno\files -RequireLlama

# 2. Compile with Inno Setup 6 (ISCC on PATH)
ISCC.exe installer\inno\WinServeAI.iss
# Output: dist\WinServeAI-0.1.0-setup.exe
```

## Script notes

| Section | Behavior |
| --- | --- |
| `[Files]` | Installs staged `files\` tree; `config\default.yaml` uses `onlyifdoesntexist` |
| `[Icons]` | Start Menu → `winserve-tray.exe` if present, else `winserve.exe`; never `llama-server` |
| `[Tasks]` `desktopicon` | Optional desktop shortcut (default off) |
| `[Tasks]` `lanfirewall` | Optional inbound rule for `{app}\bin\llama-server.exe` (default off) |
| `[UninstallRun]` | Deletes the same firewall rule name |

See [docs/specs/F-installer.md](../../docs/specs/F-installer.md).
