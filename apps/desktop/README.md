# WinServeAI tray (`winserve-tray`)

Tauri 2 desktop shell. **All lifecycle goes through `ServerManager`** in the
`winserve` crate — the webview never spawns `llama-server`.

```text
webview → Tauri commands → ServerManager → runtime → bin/llama-server.exe → /v1
```

## Develop

From repo root (config + `bin/` resolved relative to cwd):

```bash
cd apps/desktop
npm install
npm run dev
```

Requires a Tauri 2 host toolchain (Rust + platform webview). Binary name:
`winserve-tray`.

## Commands (E1a / E1b)

| Command | Manager |
| --- | --- |
| `manager_status` | `get_status` + `openai_base` |
| `manager_start` | `start` |
| `manager_stop` | `stop` |
| `manager_restart` | `restart` |
| `manager_endpoint` | `openai_base` |
| `manager_config_summary` | read settings DTO (+ `editable` flag) |
| `manager_apply_settings` | validate + write YAML via `ServerManager::apply_config` |
| `pick_model_path` | native dialog, `.gguf` only (no download) |
| `manager_logs` | viewer-only tail of `server` / `llama` / `error` logs |

### Status UI (E1b)

Badge labels are exactly: **Stopped**, **Starting**, **Ready**, **Failed**,
**Stopping**, **Crashed**. There is no “Running” alias for Ready. Buttons disable
during transitional states; Start optimistically shows Starting while
`manager_start` awaits readiness.

### Log viewer (E1c)

Tabs for `server.log` / `llama.log` / `error.log`; tails via
`winserve::server::logs::tail_dir` (timestamps already on each line). Read-only.

### Settings (E1d) + model path picker (E2)

Form edits `server.host` / `server.port` / `model.path` / `gpu.*` /
`runtime.context` / `runtime.flash_attention`. Save is rejected while
Starting / Ready / Stopping.

**Browse…** opens a native file dialog filtered to `.gguf` (`pick_model_path`);
no downloads. Persist with Save settings.

### System tray

Notification-area icon with menu: **Open**, **Start**, **Stop**, **Restart**,
**Settings**, **Quit**. Start/stop/restart call `ServerManager` only. Tooltip
shows status + OpenAI `/v1` endpoint.

- **Window close** hides to tray (does not stop the backend).
- **Quit** (tray menu or OS exit) calls `ServerManager::stop()` before leaving.
  Job Object (`KILL_ON_JOB_CLOSE`) remains the orphan backstop if force-killed.
- Lockfile + IPC pipe stay held so `winserve status|stop|restart` can attach.

## See also

- [docs/specs/E-desktop.md](../../docs/specs/E-desktop.md)
- [docs/TODO.md](../../docs/TODO.md) (E1a)
