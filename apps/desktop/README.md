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
| `manager_config_summary` | read-only config fields |
| `manager_logs` | viewer-only tail of `server` / `llama` / `error` logs |

### Status UI (E1b)

Badge labels are exactly: **Stopped**, **Starting**, **Ready**, **Failed**,
**Stopping**, **Crashed**. There is no “Running” alias for Ready. Buttons disable
during transitional states; Start optimistically shows Starting while
`manager_start` awaits readiness.

### Log viewer (E1c)

Tabs for `server.log` / `llama.log` / `error.log`; tails via
`winserve::server::logs::tail_dir` (timestamps already on each line). Read-only.

Settings, path picker, and quit→stop are E1d–E1e / E2.

## See also

- [docs/specs/E-desktop.md](../../docs/specs/E-desktop.md)
- [docs/TODO.md](../../docs/TODO.md) (E1a)
