# WinServeAI completion + test-gap audit

**Date:** 2026-07-30  
**Scope:** MVP completion + test gaps after G–I merge to `dev`  
**Branch:** `dev`

---

## Audit basis

- `dev` is checked out at `805f908`, matches `origin/dev`, and contains the merged G–I work through I4.
- This was a read-only source, test, CI, installer, and readiness-scorecard review.
- No test suite or Windows operator smoke was run as part of this audit.
- “Implemented” below means the product path exists in source.
- “Verified/proven on Windows” requires Windows execution evidence; source presence or compilation alone does not prove the complete product path.

---

## Implemented product surfaces

### CLI serve, start, and attach

- `app/src/main.rs` implements:
  - `serve`: acquires the resident lock, starts IPC, owns one `ServerManager`, attempts backend start, and remains resident.
  - `start`: one-shot foreground start, waits for readiness, then stops on Ctrl+C.
  - `status`, `stop`, and `restart`: attach to the resident owner through the lockfile-recorded IPC endpoint.
  - `print-config` and `print-cmd`: inspect resolved YAML and llama.cpp argv.
- `status` without a resident owner reports `Stopped` and the configured endpoint.
- IPC supports `start`, but the CLI’s `start` command is deliberately the one-shot path rather than an attached resident command.

### IPC, lockfile, and single instance

- `app/src/ipc/lockfile.rs` implements:
  - `%LOCALAPPDATA%\WinServeAI\manager.lock` on Windows.
  - PID and pipe-name persistence.
  - Exclusive acquisition, stale-PID reclamation, release-on-drop, and second-owner rejection.
- `app/src/ipc/pipe.rs` implements:
  - Windows named pipe `\\.\pipe\winserve-manager`.
  - JSON requests for `status`, `start`, `stop`, `restart`, `health`, and `endpoint`.
  - A non-Windows local transport for development and tests.
- Both `winserve serve` and `winserve-tray` acquire the same lock, providing the implemented single-instance boundary.

### ServerManager

- `app/src/server/manager.rs` remains the sole orchestrator.
- Implemented lifecycle:
  - Load and validate YAML.
  - Detect hardware.
  - Build argv through `app/src/runtime/llama.rs`.
  - Reject missing binary, empty or missing model path, and unavailable port.
  - Spawn the backend and wait for readiness.
  - Transition through canonical `Stopped`, `Starting`, `Ready`, `Stopping`, `Failed`, and `Crashed` states.
  - Stop with an eight-second grace period and force-kill fallback.
  - Restart through stop followed by start.
  - Detect an exited child when status is polled.
- `app/src/server/resident.rs` serializes resident IPC commands through the same manager instance.
- `app/src/server/health.rs` polls `/v1/models` first and `/health` as a fallback.
- `app/src/server/logs.rs` writes the three log streams, timestamps records, rotates by size or age, and supports log tailing.

### Runtime and Windows Job Object

- `app/src/runtime/llama.rs` is the sole owner of raw llama.cpp flags.
- `app/src/runtime/process.rs` implements:
  - Captured stdout and stderr.
  - Windows `CREATE_NEW_PROCESS_GROUP | CREATE_SUSPENDED`.
  - Job Object assignment with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` before resuming the child.
  - `CTRL_BREAK_EVENT`, grace wait, then force kill.
  - Job-handle close on drop as the orphan-reaping backstop.

### Desktop/tray shell

- `apps/desktop/src-tauri/src/lib.rs` implements the `winserve-tray` Tauri shell.
- Its commands call `ServerManager` for status, start, stop, restart, endpoint, settings, model selection, and log viewing.
- It owns the same lockfile and IPC endpoint as `winserve serve`, allowing CLI attachment.
- Window close and application exit call `ServerManager::stop()` before exit.
- The local model picker restricts selection to an existing `.gguf`.
- The current surface is still a window-oriented shell: `docs/readiness/ui.md` records that a system-tray icon/menu is not implemented.

### Operational scripts

- `scripts/start.ps1`, `scripts/stop.ps1`, and `scripts/reset.ps1` provide Windows development operations.
- `scripts/stop.ps1` prefers graceful IPC stop when a live resident lock exists and retains force/name-based fallback behavior.
- `scripts/smoke-openai.ps1` implements the A2 readiness and OpenAI request smoke flow.
- `scripts/smoke-check.sh` runs host-safe preflight checks without requiring a GGUF.
- `scripts/fetch-llama-pin.ps1` and `scripts/fetch-llama-pin.sh` implement pinned llama.cpp acquisition support.
- `scripts/check.sh` and `scripts/check_doc_drift.py` provide repository validation and documentation-drift checks.

### Release staging

- `scripts/stage-release.ps1` and `scripts/stage-release.sh` stage:
  - `winserve`
  - Optional `winserve-tray`
  - `bin/llama-server` and sibling DLLs when present
  - `config/default.yaml`
  - `FIRST_RUN.txt`
  - `notices/`
  - `logs/`
- Both scripts can require the llama binary for installer-oriented staging.
- No automated test currently asserts the resulting tree, required-file failures, pin record, or DLL/notices copying.

### Inno Setup

- `installer/inno/WinServeAI.iss` implements:
  - x64-compatible installation under Program Files.
  - Manager, optional tray, backend, config, first-run guidance, notices, and logs payloads.
  - Start Menu and optional desktop shortcuts to the tray/manager, never directly to `llama-server.exe`.
  - Optional LAN firewall creation for `llama-server.exe`.
  - Firewall-rule deletion during uninstall.
  - Preservation of an existing operator config during upgrade.
- The script has not been compiled by CI, and no clean-machine install smoke is recorded.

---

## Implemented versus proven on Windows

### Implemented

- Resident `serve`, foreground `start`, and resident attach commands.
- Shared lockfile, named-pipe IPC, stale-lock cleanup, and single-instance rejection.
- `ServerManager` lifecycle and readiness state machine.
- Suspended spawn, Job Object assignment, CTRL_BREAK, and force-kill fallback.
- Desktop shell ownership, settings, `.gguf` picker, logs, and graceful close.
- PowerShell operations and A2 smoke scripts.
- Release-layout staging.
- Inno Setup packaging, shortcuts, firewall behavior, notices, and first-run guidance.

### Verified or partially exercised by repository automation

- `.github/workflows/ci.yml` runs on `windows-latest` for pushes and pull requests targeting `main` or `dev`.
- CI runs:
  - `cargo check -p winserve`
  - `cargo test -p winserve`
  - `python scripts/check_doc_drift.py`
- The Windows Rust test build exercises Windows-compiled paths in the `winserve` package.
- `app/src/runtime/process.rs` has a Windows spawn-and-stop test using `cmd /C ping`.
- `app/src/ipc/pipe.rs` has a local transport round trip that uses the named-pipe implementation on Windows.
- Lockfile, configuration, readiness polling, argv construction, logs, network probing, memory detection, and manager settings writes have unit coverage.

### Not yet proven on Windows

- A2 with the pinned `llama-server.exe`, a real local GGUF, `/v1/models`, and an OpenAI request.
- Ctrl+C and manager-force-kill both leaving no orphan `llama-server.exe`.
- Clean install → first-run model selection → `Ready`.
- End-to-end named-pipe CLI attach and second-instance rejection using real processes.
- Job Object cleanup after the manager process itself is killed.
- Desktop-shell close/quit while `Ready`.
- Full `ServerManager` lifecycle against a controllable backend.
- PowerShell script behavior under automated tests.
- Release-layout assertions on Windows.
- Successful `ISCC.exe` compilation, install, upgrade, uninstall, shortcuts, or firewall cleanup.
- DXGI adapter results on known Windows GPU hardware.

---

## Existing automated test coverage

There are 27 test functions under `app/src/**`.

Covered modules: `server/config.rs`, `server/health.rs`, `server/logs.rs`, `server/manager.rs` (apply_config only), `runtime/llama.rs`, `runtime/process.rs` (spawn/stop sleep), `ipc/lockfile.rs`, `ipc/pipe.rs` (local transport), `system/{network,gpu,memory}.rs`.

No automated tests exist in: `app/src/main.rs`, `app/src/server/resident.rs`, `app/src/api/openai.rs`, `apps/desktop/src-tauri/`, `scripts/stage-release.*`, operational PowerShell scripts, or `installer/inno/WinServeAI.iss`.

### CI coverage boundary

- `.github/workflows/ci.yml` tests only package `winserve`.
- It does not build or test `winserve-tray`.
- It does not run `scripts/smoke-openai.ps1`, stage-release, or `ISCC.exe`.
- It does not exercise a real `llama-server.exe` or GGUF.

---

## Missing tests, prioritized

### P0 — MVP proof blockers

1. **A2 Windows + GGUF smoke** — pin `b9866`, real `.gguf`, `scripts/smoke-openai.ps1`, Ready + `/v1/models` + OpenAI request.
2. **Orphan-free quit** — Ctrl+C and force-kill manager leave no `llama-server.exe` (`start`, `serve`, tray).
3. **Install → Ready** — ISCC compile, clean install, Browse `.gguf`, Ready; shortcuts + uninstall firewall.

### P1 — Ownership and lifecycle integration

1. **Named-pipe attach and single instance** — real resident + CLI attach + second-owner refusal.
2. **Job Object manager-kill** — kill manager only; child exits.
3. **Tray quit** — close while Ready; CLI attach to tray owner.
4. **ServerManager lifecycle** — missing binary/model, busy port, timeout, crash detection (fake backend).

### P2 — Packaging verification

1. **Stage-release** tree/required-binary/notices/`VERSION` asserts.
2. **ISCC** compile job + optional install smoke.

### P3 — Lower-risk unit and platform coverage

CLI exit codes, `api/openai.rs` URL helpers, PowerShell Pester, DXGI on known-GPU runners.

---

## Evidence map

| Priority / gap | Readiness evidence | Proposed test location |
| --- | --- | --- |
| P0 A2 Windows + GGUF | `docs/readiness/runtime-llama.md`, `server-manager.md`, `bin.md` | `scripts/smoke-openai.ps1` + `tests/windows/a2-smoke.ps1` |
| P0 orphan-free quit | `runtime-process.md`, `server-manager.md`, `ui.md` | `app/tests/orphan_reaping_windows.rs`, `tests/windows/tray-quit.ps1` |
| P0 install → Ready | `installer.md`, `ui.md` | `tests/windows/install-ready.ps1` |
| P1 named-pipe attach / single instance | `cli.md`, `server-manager.md` | `app/tests/resident_attach_windows.rs` |
| P1 Job Object manager-kill | `runtime-process.md` | `app/tests/job_object_windows.rs` |
| P1 tray quit / tray-owned attach | `ui.md`, `cli.md` | `apps/desktop/src-tauri/tests/tray_lifecycle_windows.rs` |
| P1 manager lifecycle | `server-manager.md`, `server-health.md` | `app/tests/server_manager_lifecycle.rs` |
| P2 stage-release | `installer.md` | `tests/scripts/stage-release.Tests.ps1` |
| P2 ISCC compile | `installer.md` | CI packaging job + `tests/windows/installer-smoke.ps1` |
| P3 CLI / API / PS1 / DXGI | `cli.md`, `api.md`, `scripts-ops.md`, `system.md` | unit/Pester/`dxgi_windows.rs` |

Proposed paths are recommendations, not existing files.

---

## Documentation drift found

- `docs/TODO.md` marks G–I complete (current backlog truth).
- `docs/PLAN.md` previously said F “Not started” and left ship checklist unchecked — reconciled in the same change set as this audit.
- Readiness scores are unchanged; this audit does not invent maturity bumps.
- Remaining open proof: A2, orphan-free quit, install→Ready (see Verdict).

---

## Findings

- G–I implementation surfaces are present on `dev`.
- Architecture holds: one llama.cpp backend, one `ServerManager`, raw flags only in `runtime/llama.rs`.
- Rust unit coverage is meaningful for config/readiness/logs/argv/lockfile/IPC/process/probes.
- CI is Windows-based but limited to `winserve` + doc drift.
- Largest remaining risk is missing end-to-end Windows proof, not missing implementation.
- Job Object / named-pipe unit tests do not prove manager-death reaping or real-process CLI/tray ownership.
- Installer authored but uncompiled/uninstalled by automation.
- Tray binary exists; true system-tray icon/menu still absent (`docs/readiness/ui.md`).

---

## Verdict

| Check | Result |
| --- | --- |
| Branch is `dev` and G–I merges are present | **PASS** |
| CLI serve/start/attach implemented | **PASS** |
| IPC, lockfile, and single-instance implementation present | **PASS** |
| ServerManager lifecycle implementation present | **PASS** |
| Windows Job Object implementation present | **PASS** |
| Desktop manager shell and graceful close implemented | **PASS** |
| Release staging and Inno sources implemented | **PASS** |
| Core Rust unit coverage present | **PASS** |
| Windows CI configured for `winserve` check/test | **PASS** |
| A2 proven with pinned llama.cpp and real GGUF | **NO** |
| Orphan-free manager/tray termination proven | **NO** |
| Named-pipe CLI/tray ownership proven end to end | **NO** |
| Full ServerManager lifecycle covered | **NO** |
| Desktop shell built and tested in CI | **NO** |
| Stage-release payload tested automatically | **NO** |
| Inno script compiled in CI | **NO** |
| Clean Windows install → Ready proven | **NO** |
| MVP implementation complete enough for focused Windows acceptance testing | **PASS** |
| MVP completion proven for release | **NO** |
