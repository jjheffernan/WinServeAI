# Research: development.md

**Target:** `docs/development.md`  
**Branch / layout:** `refactor/minimal-appliance` — single crate, no `packages/` monorepo.

## Problem with current stub

`docs/development.md` is thin and only half-aligned:

* Mentions `cargo -p winserve` (correct) but omits `status`, `print-config`, and MVP stop behavior.
* Scripts listed; no behavior notes (`start.ps1` prefers release then debug; `stop.ps1` force-kills by process name).
* Architecture pointer is fine; needs an explicit **repo layout** and **do not reintroduce `packages/`** rule.

## Ground truth (code)

### Workspace

| Fact | Source |
| --- | --- |
| Workspace members: `["app"]` only | root `Cargo.toml` |
| Package / bin name: `winserve` | `app/Cargo.toml` (`[[bin]] name = "winserve"`) |
| No `packages/*` crates in this tree | layout + `AGENTS.md` |

Old multi-crate layout (`packages/launcher`, `packages/config`, …) lives only under example worktrees / history — **not** the product tree.

### CLI (`app/src/main.rs`)

| Command | Behavior |
| --- | --- |
| `start` (default if no args) | Load config → `ServerManager::start` → print `READY <openai base>` → block on Ctrl+C → `stop` |
| `status` | Load config, print `Status` + OpenAI base URL (no long-running manager → usually `Stopped`) |
| `print-config` | Load YAML, print serialized config |
| `print-cmd` | Load config, print `binary argv…` from `build_command()` |
| `stop` / `restart` | Exit failure; message says use Ctrl+C on `winserve start` (tray/service later) |

Config path:

* Default: `<cwd>/config/default.yaml` (`repo_root()` = `current_dir()`)
* Override: `WINSERVE_CONFIG`

Missing config → error: copy `config/default.yaml` and set `model.path`.

### External binary

* Resolved as `<repo_root>/bin/llama-server.exe` on Windows, `bin/llama-server` elsewhere (`runtime/llama.rs::default_binary`)
* Spawn workdir = binary parent (`bin/`) so CUDA/runtime DLLs beside the exe resolve
* Start fails if binary or `model.path` missing

### Config

* Source of truth: `config/default.yaml` (see `docs/configuration.md`)
* Sections: `server`, `model`, `gpu`, `runtime`, `logging`
* No raw llama flags in YAML; argv only in `app/src/runtime/llama.rs`

### Scripts

| Script | Role |
| --- | --- |
| `scripts/start.ps1` | `cd` repo root, set `WINSERVE_CONFIG` to `config\default.yaml`, run `target\release\winserve.exe start`, else `target\debug\...` |
| `scripts/stop.ps1` | `Stop-Process -Force` on `winserve` and `llama-server` |
| `scripts/reset.ps1` | `stop.ps1` + delete files under `logs/` (keeps config/models) |
| `scripts/check.sh` | `cargo check -p winserve` + `cargo test -p winserve` |

### App layout (`app/`)

```text
app/
  Cargo.toml          # package winserve
  src/
    main.rs           # CLI
    lib.rs            # modules + ServerManager re-export
    server/           # manager, config, health, logs
    runtime/          # llama argv + process spawn
    system/           # gpu, memory, network
    api/              # OpenAI URL helpers (passthrough)
```

`ui/` reserved later (empty in MVP per architecture).

### CI

`.github/workflows/ci.yml`: `windows-latest`, `cargo check -p winserve`, `cargo test -p winserve`.

## What development.md must cover

1. Prerequisites (Rust stable, Windows primary, `bin/llama-server.exe`, GGUF at `model.path`)
2. Build / check / test via `-p winserve`
3. CLI: `print-cmd`, `print-config`, `start`, `status`; stop via Ctrl+C or `stop.ps1`
4. Config path + `WINSERVE_CONFIG`
5. PowerShell scripts and `check.sh`
6. Repo + `app/` layout; **no `packages/` monorepo**
7. Links: architecture, configuration, logging — no duplicate schema/flag tables

## Out of scope for this doc

* Installer packaging details → `installer.md`
* Full YAML schema → `configuration.md`
* Log file semantics → `logging.md`
* Multi-backend / Tauri / chat UI

## Sources

* `app/src/main.rs`, `app/src/server/manager.rs`, `app/src/runtime/llama.rs`
* `app/Cargo.toml`, root `Cargo.toml`
* `scripts/*.ps1`, `scripts/check.sh`
* `config/default.yaml`, `bin/README.md`
* `docs/architecture.md`, `AGENTS.md`, `README.md`
* `docs/research/doc-build/PLAN.md`
