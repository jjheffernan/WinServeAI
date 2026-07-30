# Development

> **Readiness:** cli 2.6/5 (`mvp-partial`), scripts-ops 3.6/5 (`mvp-ready`) — details in [readiness/cli.md](./readiness/cli.md), [readiness/scripts-ops.md](./readiness/scripts-ops.md)

WinServeAI is a **single Rust crate** (`winserve` in `app/`) that owns process lifecycle for an external `bin/llama-server.exe`. There is no `packages/` monorepo and no backend-trait workspace.

See [architecture.md](architecture.md) for the runtime boundary and rules.

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
app/                 # only workspace member — package name winserve
  src/main.rs        # CLI
  src/lib.rs
  src/server/        # ServerManager, config, health, logs
  src/runtime/       # llama argv + process spawn (external boundary)
  src/system/        # GPU / memory / network probes
  src/api/           # OpenAI URL helpers (passthrough)
bin/                 # llama-server.exe (not built by this repo)
config/              # default.yaml
logs/                # server.log, llama.log, error.log (created at runtime)
models/              # your GGUF files (not shipped)
installer/
scripts/             # start/stop/reset.ps1, smoke-openai.ps1, smoke-check.sh, check.sh
docs/
```

Root `Cargo.toml` is a one-member workspace:

```toml
[workspace]
members = ["app"]
```

Do **not** reintroduce `packages/config`, `packages/launcher`, or other package-per-concern crates without a concrete second backend.

## Build / check / test

From the repo root:

```bash
cargo check -p winserve
cargo test -p winserve
cargo build -p winserve --release
```

Local gate (same as CI intent):

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

Minimum edit before first start: set `model.path` to a real GGUF. Schema and mapping rules: [configuration.md](configuration.md). Logs: [logging.md](logging.md).

## CLI

Binary name: `winserve`. Default command is `start` if you pass no subcommand.

```bash
cargo run -p winserve -- print-cmd
cargo run -p winserve -- print-config
cargo run -p winserve -- status
cargo run -p winserve -- start
```

| Command | Purpose |
| --- | --- |
| `print-cmd` | Print the resolved `llama-server` path and argv (no spawn). Use this to verify config → flags before starting. |
| `print-config` | Load and print the effective YAML config. |
| `status` | Print manager status and OpenAI base URL. Without a long-running manager process this is usually `Stopped`. |
| `start` | Spawn `bin/llama-server.exe`, wait until `/v1/models` is ready, print `READY http://host:port/v1`, then block until Ctrl+C. |

`stop` and `restart` are not implemented on the CLI for MVP (they need a long-running tray/service owner). Stop a foreground `start` with **Ctrl+C**, or use `scripts/stop.ps1`.

Release binary (what the PowerShell scripts prefer):

```text
target/release/winserve.exe
```

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

1. Place `llama-server.exe` (**b9866**) in `bin/`.
2. Edit `config/default.yaml` → `model.path`.
3. `cargo run -p winserve -- print-cmd` — confirm argv.
4. `cargo run -p winserve -- start` — wait for `READY http://127.0.0.1:8080/v1`.
5. `.\scripts\smoke-openai.ps1` (or any OpenAI-compatible client at that base URL).
6. Ctrl+C to free the GPU (cooperative stop).

## Architecture reminder

```text
CLI (winserve) → ServerManager (app/server) → runtime/llama + runtime/process
                                              → bin/llama-server.exe → /v1
```

Raw llama.cpp flags exist only in [`app/src/runtime/llama.rs`](../app/src/runtime/llama.rs). UI and config must not invent flags.

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — runtime boundary and rules
- [configuration.md](./configuration.md) — YAML schema and `WINSERVE_CONFIG`
- [api.md](./api.md) — readiness (`GET /v1/models`) and client examples
- [logging.md](./logging.md) — `logs/` streams
- [contributing.md](./contributing.md) — PR and branch rules
- [release-process.md](./release-process.md) — `dev` vs `main`, pin policy
- [PLAN.md](./PLAN.md) — ordered implementation milestones
- [policies/doc-drift.md](./policies/doc-drift.md) — `python3 scripts/check_doc_drift.py`
- [bin/README.md](../bin/README.md) — place `llama-server.exe`
- [`app/src/server/health.rs`](../app/src/server/health.rs) — readiness probe

### Upstream

- [llama.cpp releases (`b####`)](https://github.com/ggml-org/llama.cpp/releases) — pin source for `bin/`
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
