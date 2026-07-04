# E — Desktop shell (Phase 2 build spec)

**Milestone:** [PLAN.md](../PLAN.md) E1–E3 · **Phase:** [roadmap.md](../roadmap.md) Phase 2  
**Status:** Spec only — no Tauri/UI implementation in this slice.  
**Research:** [research/05-server-manager.md](../research/05-server-manager.md), [architecture.md](../architecture.md)

## Goal

Ship a thin Windows desktop shell so basic use needs **no CLI**: start / stop / status / logs / settings, plus a local model **path** picker. All lifecycle goes through **`ServerManager` only**.

```text
Desktop UI (webview / tray)
       │  Tauri commands / IPC only
       ▼
  ServerManager     (app/src/server/manager.rs)
       │
       ▼
  runtime → bin/llama-server.exe → /v1
```

## Non-goals (do not implement in Phase 2)

* Chat UI, conversation history, or any inference client inside the app
* Model downloads, marketplace, HuggingFace, or “pull” flows
* Backend traits, plugins, multi-provider, or alternate runtimes
* Spawning `llama-server` from the webview, Tauri sidecar, or shell plugin
* Raw llama flags in UI or config UI (flags stay in `app/src/runtime/llama.rs`)
* Docker, auth, remote management, metrics dashboard (Phase 5+)
* Auto-restart as default product behavior

## E1 — UI surfaces (ServerManager only)

### Surfaces

| Surface | User action | Manager API |
| --- | --- | --- |
| **Start** | Button / tray “Start” | `start()` → Starting → Ready \| Failed |
| **Stop** | Button / tray “Stop” | `stop()` → Stopping → Stopped (idempotent) |
| **Status** | Always-visible badge / tray tooltip | `status()` / `get_status()`, `health()`, `endpoint()` / `openai_base()` |
| **Logs** | Scrollable viewer (server + llama + error) | Read via manager/log helpers (`app/src/server/logs.rs`); UI is a **viewer only** |
| **Settings** | Form bound to YAML fields | `config()` / load + validate + write YAML; reject unsafe edits while Starting/Ready/Stopping (require stop first) |

### UI rules

1. Webview **never** calls process/llama internals or HTTP readiness as source of truth.
2. Show **Starting** until manager reports **Ready** (process alive ≠ Ready).
3. Display OpenAI base URL from manager (`http://{host}:{port}/v1`) for copy-paste into external clients.
4. Optional: hardware inventory via `hardware()` — display only, no layer math.
5. On app quit: call `stop()`; Job Object remains the backstop if force-killed ([backend.md](../backend.md)).

### Suggested stack

**Tauri 2** (or current stable Tauri) with Rust commands that hold / talk to one `ServerManager`. Prefer sharing the same crate (`winserve` lib) rather than duplicating lifecycle. Prior art: Jan-style lifecycle in Rust core, not webview ([research/05](../research/05-server-manager.md)).

CLI remains available for headless/dev; desktop is not a second orchestrator.

## E2 — Model path picker

* Native file dialog: select an existing **`.gguf`** (or any file; validate extension/path).
* Persist to YAML `model.path` through config write path (same as Settings).
* **No** download, URL fetch, or registry.
* If path missing on disk, surface the same class of error/warning as CLI config validation.
* Changing model path while Ready requires **stop**, then start (no hot-swap in Phase 2).

## E3 — Long-running manager (options + recommendation)

Today CLI `stop` / `restart` fail with “requires a long-running manager process” (`app/src/main.rs`). Desktop must fix ownership so stop/restart work from UI (and preferably CLI).

### Option A — Tray-owned only

| | |
| --- | --- |
| **Idea** | Tray/desktop process embeds `ServerManager` in-process. UI buttons call it directly. |
| **Pros** | Smallest code; no IPC protocol; matches “UI holds the manager”. |
| **Cons** | CLI `stop`/`restart`/`status` still cannot attach; two starts (CLI + tray) can conflict on port. |
| **Fit** | UI-only MVP if CLI attach is deferred. |

### Option B — Lockfile + IPC (headless manager)

| | |
| --- | --- |
| **Idea** | Dedicated manager process (`winserve serve` or tray without UI requirement). Lockfile records PID + IPC endpoint (named pipe on Windows). CLI and UI are thin clients. |
| **Pros** | CLI and UI share one owner; Ollama-like; clean single-instance. |
| **Cons** | More surface (protocol, versioning, stale lockfile); need careful Job Object lifetime (manager process owns the job, not each client). |

### Option C — Tray-owned **plus** minimal IPC (recommended)

| | |
| --- | --- |
| **Idea** | Phase 2 tray **is** the long-running owner (embeds `ServerManager` + Job Object). Expose a **minimal local IPC** (Windows named pipe) and a **lockfile** so CLI `status` / `stop` / `restart` attach to that process. Single-instance: second tray/CLI start fails with “already running”. |
| **Pros** | Satisfies E3 for both UI and CLI; one process owns children; no separate headless daemon required for MVP. |
| **Cons** | Slightly more than pure tray-owned; must define a tiny command set (`status`, `stop`, `restart`, `start` if stopped). |

**Recommendation: Option C** for Phase 2.

* Defer a fully headless Windows service to Phase 5.
* IPC is localhost-only, no auth beyond “same user session” for MVP.
* Lockfile path: e.g. `%LOCALAPPDATA%\WinServeAI\manager.lock` (PID + pipe name).
* Stale lock: if PID dead, remove lock and start fresh.

### State machine (unchanged)

Canonical states from [research/05](../research/05-server-manager.md): Stopped, Starting, Ready, Failed, Stopping, Crashed. UI labels must match; do not invent “Running” as Ready.

## Acceptance criteria

1. User can start and stop the API from the desktop UI without opening a terminal.
2. Status shows Starting vs Ready vs Failed/Crashed correctly (Ready only after manager readiness).
3. Log viewer shows lines from `logs/` streams (or live forward from manager) with timestamps.
4. Settings edit `server.host` / `server.port` / `model.path` / performance fields and persist YAML; invalid values rejected with clear messages.
5. Model path picker sets `model.path` only (no download UI).
6. Quit or Stop leaves no orphan `llama-server` (Job Object + graceful stop).
7. With tray running, `winserve stop` / `winserve status` work via IPC (Option C).
8. No chat UI, no model download entry points in the shipped UI.
9. UI code paths do not reference llama argv flags or spawn `llama-server` directly.

## File touch list (expected implementation)

| Path | Change |
| --- | --- |
| `app/src/server/manager.rs` | Stable API for UI/IPC; possibly async event hooks |
| `app/src/server/config.rs` | Settings validation already present; wire update path |
| `app/src/server/logs.rs` | Log subscription / tail for UI |
| `app/src/main.rs` | CLI attaches via lockfile+IPC for `stop`/`restart`/`status`; optional `serve` if split |
| `app/src/ipc/` *(new)* | Named pipe server/client, lockfile helpers (Windows-first) |
| `app/ui/` or `src-tauri/` *(new)* | Tauri project: commands wrap manager/IPC only |
| `app/ui/**` frontend | Start/stop/status/logs/settings + path picker |
| `config/default.yaml` | Unchanged schema; UI writes same keys |
| `docs/development.md`, `docs/architecture.md` | Document tray + IPC after implementation |
| `docs/PLAN.md` | Check E1–E3 when done |
| `installer/` | Shortcut target may become tray exe (coordinate with [F-installer.md](F-installer.md)) |

Do **not** add `app/src/runtime/llama.rs` flags from UI code. Do **not** introduce backend traits.

## Implementation order (when coding)

1. Lockfile + named pipe + CLI attach (`stop`/`status`/`restart`) against an in-process manager in a long-lived `winserve` mode.
2. Tauri shell: commands → manager (in-process in tray binary).
3. Status + start/stop + log viewer.
4. Settings + model path picker.
5. Single-instance + quit → `stop()`.

## See also

- [PLAN.md](../PLAN.md) — E1–E3 checkboxes
- [research/05-server-manager.md](../research/05-server-manager.md) — public API and UI must-nots
- [architecture.md](../architecture.md) — appliance rules
- [F-installer.md](F-installer.md) — Phase 3 packages the tray/manager
- [A2-smoke.md](A2-smoke.md) — prove engine before desktop polish
