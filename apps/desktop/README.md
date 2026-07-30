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

## Commands (E1a)

| Command | Manager |
| --- | --- |
| `manager_status` | `get_status` + `openai_base` |
| `manager_start` | `start` |
| `manager_stop` | `stop` |
| `manager_restart` | `restart` |
| `manager_endpoint` | `openai_base` |
| `manager_config_summary` | read-only config fields |

UI polish (canonical badges, logs, settings, path picker, quit path) is E1b–E1e / E2.

## See also

- [docs/specs/E-desktop.md](../../docs/specs/E-desktop.md)
- [docs/TODO.md](../../docs/TODO.md) (E1a)
