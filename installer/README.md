# Installer

Native Windows installer (Inno Setup preferred).

## Goals

* Install `winserve.exe` + `winserve-tray.exe` + `bin/llama-server.exe`
* Desktop / Start Menu shortcut → tray (preferred) or manager
* Firewall rule only when bind is non-loopback
* `config/default.yaml` + `notices/THIRD_PARTY_NOTICES.md`

## Stage release layout (F0)

From a Windows build host (or any host that has release artifacts):

```powershell
# Build + stage into dist\WinServeAI
.\scripts\stage-release.ps1

# Or stage into the Inno files tree (F1 input)
.\scripts\stage-release.ps1 -OutDir installer\inno\files
```

```bash
./scripts/stage-release.sh --out dist/WinServeAI
./scripts/stage-release.sh --out installer/inno/files
```

Requires `cargo build -p winserve` / `winserve-tray` artifacts under `target/release`
(unless `-SkipBuild` / `--skip-build`). Fetch the pinned llama binary with
`scripts/fetch-llama-pin.ps1` (or `.sh`) before staging if you need `bin/`.

## Layout on disk (installed / staged)

```text
%ProgramFiles%\WinServeAI\          # {app}
  winserve.exe
  winserve-tray.exe
  bin\
    llama-server.exe
    (CUDA DLLs beside exe when shipping CUDA builds)
  config\
    default.yaml
  logs\
  notices\
    THIRD_PARTY_NOTICES.md
```

## Next (F1+)

```powershell
.\scripts\stage-release.ps1 -OutDir installer\inno\files -RequireLlama
ISCC.exe installer\inno\WinServeAI.iss
```

* [`installer/inno/WinServeAI.iss`](inno/WinServeAI.iss) — Inno Setup 6 script (Files, Icons, optional desktop + LAN firewall tasks)
* First-run model path guidance (F5); notices payload polish (F4)
