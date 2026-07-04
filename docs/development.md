# Development

> **Readiness:** cli 2.6/5 (`mvp-partial`), scripts-ops 2.8/5 (`mvp-partial`) — details in [readiness/cli.md](./readiness/cli.md), [readiness/scripts-ops.md](./readiness/scripts-ops.md)

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

* Rust stable
* Windows 10/11 (primary target; CI runs on `windows-latest`)
* `bin/llama-server.exe` from a pinned llama.cpp release (see [bin/README.md](../bin/README.md))
* A GGUF model path set in config (`model.path`)

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
scripts/             # start.ps1, stop.ps1, reset.ps1, check.sh
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
.\scripts\start.ps1   # WINSERVE_CONFIG=config\default.yaml; release winserve, else debug
.\scripts\stop.ps1    # force-stop processes named winserve and llama-server
.\scripts\reset.ps1   # stop.ps1 + clear files under logs/ (keeps config and models)
```

`start.ps1` expects a prior `cargo build -p winserve` (release preferred) or `cargo build -p winserve` debug fallback.

## Typical first run

1. Place `llama-server.exe` in `bin/`.
2. Edit `config/default.yaml` → `model.path`.
3. `cargo run -p winserve -- print-cmd` — confirm argv.
4. `cargo run -p winserve -- start` — wait for `READY http://127.0.0.1:8080/v1`.
5. Point any OpenAI-compatible client at that base URL.
6. Ctrl+C (or `.\scripts\stop.ps1`) to free the GPU.

## Architecture reminder

```text
CLI (winserve) → ServerManager (app/server) → runtime/llama + runtime/process
                                              → bin/llama-server.exe → /v1
```

Raw llama.cpp flags exist only in `app/src/runtime/llama.rs`. UI and config must not invent flags.
