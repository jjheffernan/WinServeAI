# Development

> **Readiness:** cli 3.2/5 (`mvp-partial`), scripts-ops 3.4/5 (`mvp-partial`) — details in [readiness/cli.md](./readiness/cli.md), [readiness/scripts-ops.md](./readiness/scripts-ops.md)

WinServeAI owns process lifecycle for an external `bin/llama-server.exe` via
**`ServerManager`**. The manager crate is `winserve` (`app/`); the optional
desktop shell is `winserve-tray` (`apps/desktop/`). There is no backend-trait
workspace and no `packages/` monorepo.

See [architecture.md](architecture.md) for the runtime boundary, lockfile, and
named-pipe IPC. First-run model path: [first-run.md](first-run.md).

## Branches

| Branch | Purpose |
| --- | --- |
| **`dev`** | Default working branch — experimental / preview integration |
| **`main`** | Releases only — do not develop here |

```bash
git checkout dev
git pull origin dev
```

Feature work: branch off `dev`, open PRs into `dev`. Releases: PR `dev` → `main` (see [release-process.md](release-process.md)).

## Prerequisites

* Rust stable (`rustup` default host `x86_64-pc-windows-msvc` on Windows)
* **MSVC linker** — Visual Studio **Build Tools** (or full VS) with the **Desktop development with C++** workload. Required so `link.exe` is on `PATH`. VS Code alone is **not** enough.
* Windows 10/11 (primary target; CI runs on `windows-latest`)
* `bin/llama-server.exe` from pinned llama.cpp **b9866** (see [bin/README.md](../bin/README.md))
* A GGUF model path set in config (`model.path`)

### Fix: `linker link.exe not found`

Rust’s MSVC target needs the Visual C++ toolchain. Installing Build Tools is not enough if `link.exe` is not on `PATH` in **this** terminal (Cursor’s integrated terminal often does not load VS vars).

#### 1. Confirm the C++ workload is installed

1. Open **Visual Studio Installer** → **Modify** on Build Tools 2022 (or later).
2. Enable **Desktop development with C++**.
3. On the right, ensure **MSVC v143** (or latest) and **Windows 10/11 SDK** are checked.
4. Apply, then **fully quit and reopen** Cursor (or reboot).

#### 2. Prefer a VS developer shell

From the Start menu open one of:

- **Developer PowerShell for VS 2022**, or
- **x64 Native Tools Command Prompt for VS 2022**

Then:

```powershell
cd path\to\WinServeAI
where.exe link
rustup show
cargo run -p winserve -- print-cmd
```

`where.exe link` must print a path under `...\VC\Tools\MSVC\...\link.exe`. If it still fails, the C++ workload is incomplete — go back to step 1.

#### 3. Or inject VS vars into the current PowerShell

```powershell
# Adjust year/edition if needed (BuildTools vs Community)
$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -property installationPath
cmd /c "`"$vs\VC\Auxiliary\Build\vcvars64.bat`" && set" | ForEach-Object {
  if ($_ -match '^(.*?)=(.*)$') { Set-Item -Path "env:$($matches[1])" -Value $matches[2] }
}
where.exe link
cargo run -p winserve -- print-cmd
```

#### 4. Confirm Rust target

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup show
```

Host should be `x86_64-pc-windows-msvc`, not `gnu`.

**Alternative (not recommended):** `stable-x86_64-pc-windows-gnu` needs MinGW and does not match CI. Prefer MSVC.

## Repo layout

```text
app/                       # package winserve — ServerManager + CLI
  src/main.rs              # CLI: serve | start | stop | status | restart | …
  src/server/              # manager, config, health, logs, resident loop
  src/ipc/                 # lockfile + named-pipe (Windows) / loopback TCP (dev)
  src/runtime/             # llama argv + process spawn (external boundary)
  src/system/              # GPU / memory / network probes
  src/api/                 # OpenAI URL helpers (passthrough)
apps/desktop/              # Tauri 2 shell → winserve-tray (wraps ServerManager)
bin/                       # llama-server.exe (not built by this repo)
config/                    # default.yaml (empty model.path until first run)
logs/                      # server.log, llama.log, error.log (created at runtime)
models/                    # your GGUF files (not shipped)
installer/inno/            # Inno Setup script + staged files/
scripts/                   # start/stop/reset, stage-release, smoke, check
docs/
```

Root workspace:

```toml
[workspace]
members = ["app", "apps/desktop/src-tauri"]
```

Do **not** reintroduce `packages/config`, `packages/launcher`, or other
package-per-concern crates without a concrete second backend.

## Build / check / test

From the repo root:

```bash
cargo check -p winserve
cargo test -p winserve
cargo build -p winserve --release
# Desktop shell (needs Tauri host toolchain):
cargo build -p winserve-tray --release
```

Local gate (same as CI intent for the manager crate):

```bash
./scripts/check.sh
```

## Doc checks

After readiness or primary-doc edits: `python3 scripts/check_doc_drift.py` (see [policies/doc-drift.md](policies/doc-drift.md)).

## Config

Default path: `config/default.yaml` (relative to the process current directory).

Override:

```bash
# bash / Git Bash
export WINSERVE_CONFIG=/path/to.yaml

# PowerShell
$env:WINSERVE_CONFIG = "D:\path\to.yaml"
```

Minimum edit before first start: set `model.path` to a real GGUF (shipped
default is empty — [first-run.md](first-run.md)). Schema:
[configuration.md](configuration.md). Logs: [logging.md](logging.md).

## CLI

Binary name: `winserve`. Default command is `start` if you pass no subcommand.

```bash
cargo run -p winserve -- print-cmd
cargo run -p winserve -- print-config
cargo run -p winserve -- serve          # resident owner (lockfile + IPC)
cargo run -p winserve -- status         # attach to resident, else Stopped
cargo run -p winserve -- stop
cargo run -p winserve -- restart
cargo run -p winserve -- start          # one-shot foreground (no IPC owner)
```

| Command | Purpose |
| --- | --- |
| `print-cmd` | Print resolved `llama-server` path + argv (no spawn). |
| `print-config` | Load and print the effective YAML config. |
| `serve` | **Resident owner:** acquire lockfile, listen on IPC, start backend, stay up until Ctrl+C. Second `serve`/tray refuses with “already running”. |
| `start` | One-shot: spawn backend, print READY, block until Ctrl+C — **does not** take the lock / IPC. Prefer `serve` or the tray for day-to-day. |
| `status` | Attach to resident via IPC when present; otherwise print `Stopped` + configured endpoint. |
| `stop` / `restart` | Attach to resident (`serve` or tray). Fail with a clear error if none is running. |

### Resident lock + IPC

| Item | Windows | Dev (non-Windows) |
| --- | --- | --- |
| Lockfile | `%LOCALAPPDATA%\WinServeAI\manager.lock` | `$XDG_RUNTIME_DIR` / `$TMPDIR` / `/tmp` + `WinServeAI/manager.lock` |
| Pipe name | `winserve-manager` → `\\.\pipe\winserve-manager` | Same logical name; loopback TCP + `.port` file beside the lock |
| Owners | `winserve serve` **or** `winserve-tray` | Same |

`scripts/stop.ps1` prefers `winserve stop` when the lock is live; `-Force` falls
back to process-name kill.

Release binaries (PowerShell scripts prefer release):

```text
target/release/winserve.exe
target/release/winserve-tray.exe   # or CARGO_TARGET_DIR equivalent
```

## Desktop tray (`winserve-tray`)

Tauri 2 shell under `apps/desktop/`. Webview commands call **`ServerManager`
only** — never spawn `llama-server` from JS. While running, the tray holds the
same lockfile + IPC pipe so CLI `status|stop|restart` attach.

```bash
cd apps/desktop && npm install && npm run dev
# or from repo root:
cargo run -p winserve-tray
```

Canonical status labels: Stopped / Starting / Ready / Failed / Stopping /
Crashed. Settings + `.gguf` Browse require Stopped / Failed / Crashed. Quit
calls `stop()`; Job Object remains the orphan backstop.

Details: [apps/desktop/README.md](../apps/desktop/README.md),
[specs/E-desktop.md](specs/E-desktop.md).

## External binary

`ServerManager` resolves the backend as:

```text
<repo_root>/bin/llama-server.exe   # Windows
<repo_root>/bin/llama-server       # non-Windows
```

The child process working directory is `bin/` so CUDA and other runtime DLLs beside the executable resolve. WinServeAI does not build or vendor llama.cpp source.

## Scripts

Run from anywhere; scripts locate the repo root from `scripts/`.

```powershell
.\scripts\start.ps1          # WINSERVE_CONFIG=config\default.yaml; release winserve, else debug
.\scripts\stop.ps1           # IPC graceful stop when lockfile exists; -Force for name kill
.\scripts\reset.ps1          # stop.ps1 + clear files under logs/ (keeps config and models)
.\scripts\smoke-openai.ps1   # A2: poll GET /v1/models (+ optional chat); needs binary + GGUF
.\scripts\smoke-openai.ps1 -Start   # optional: start winserve in background, then poll
```

```bash
./scripts/smoke-check.sh     # any host: cargo test + print-cmd + config preflight (no inference)
```

`start.ps1` expects a prior `cargo build -p winserve` (release preferred) or `cargo build -p winserve` debug fallback.

A2 operator checklist: [specs/A2-smoke.md](specs/A2-smoke.md). Prefer **Ctrl+C** on foreground `start` for cooperative stop; do not treat `stop.ps1` as the A2 pass path.

## Typical first run

1. Place `llama-server.exe` (**b9866**) in `bin/` (`.\scripts\fetch-llama-pin.ps1`).
2. Set `model.path` to a local `.gguf` (edit YAML or tray Settings → Browse).
3. `cargo run -p winserve -- print-cmd` — confirm argv.
4. Prefer resident owner:
   - `cargo run -p winserve -- serve`, **or**
   - `cargo run -p winserve-tray` / Start Menu shortcut after install
5. Wait for Ready / `READY http://127.0.0.1:8080/v1`.
6. `.\scripts\smoke-openai.ps1` (or any OpenAI-compatible client).
7. Stop: tray Stop / quit, `winserve stop`, or Ctrl+C on `serve`.

Foreground `winserve start` remains useful for quick one-shot debug (Ctrl+C
stop); it does not accept CLI `stop`/`restart` attach.

## Architecture reminder

```text
CLI / tray ──► ServerManager ──► runtime/llama + process ──► bin/llama-server.exe ──► /v1
     │                ▲
     └── IPC attach ──┘   (serve or tray owns lockfile + pipe)
```

Raw llama.cpp flags exist only in [`app/src/runtime/llama.rs`](../app/src/runtime/llama.rs). UI and config must not invent flags.

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — runtime boundary, lockfile, IPC
- [first-run.md](./first-run.md) — empty `model.path` → set `.gguf`
- [installer.md](./installer.md) — stage-release + Inno shortcuts → tray
- [configuration.md](./configuration.md) — YAML schema and `WINSERVE_CONFIG`
- [api.md](./api.md) — readiness (`GET /v1/models`) and client examples
- [logging.md](./logging.md) — `logs/` streams
- [apps/desktop/README.md](../apps/desktop/README.md) — tray commands
- [contributing.md](./contributing.md) — PR and branch rules
- [release-process.md](./release-process.md) — `dev` vs `main`, pin policy
- [PLAN.md](./PLAN.md) — ordered implementation milestones
- [policies/doc-drift.md](./policies/doc-drift.md) — `python3 scripts/check_doc_drift.py`
- [bin/README.md](../bin/README.md) — place `llama-server.exe`
- [`app/src/server/health.rs`](../app/src/server/health.rs) — readiness probe
- [`app/src/ipc/`](../app/src/ipc/) — lockfile + pipe

### Upstream

- [llama.cpp releases (`b####`)](https://github.com/ggml-org/llama.cpp/releases) — pin source for `bin/`
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
