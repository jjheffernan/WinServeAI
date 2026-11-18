# Ops resilience (restart / power / update)

How WinServeAI behaves when the machine restarts, power drops, or the installer upgrades an existing install. Companion to [configuration.md](configuration.md) and [release-process.md](release-process.md).

## Host / IP stays modular

| Rule | Behavior |
| --- | --- |
| Default bind | `server.host: 127.0.0.1` in shipped YAML — **localhost only** |
| No auto-pick | We never rewrite `server.host` to a detected LAN, VPN, Hyper-V, or **WSL** adapter IP on start, restart, or upgrade |
| Operator-owned | Change bind only via Settings / YAML (`127.0.0.1`, a chosen interface IP, or `0.0.0.0` for all interfaces) |
| Upgrade | Installer copies `config\default.yaml` with **`onlyifdoesntexist`** — existing host/port/key/model path are kept |
| Wildcard clients | Bind `0.0.0.0` / `::` is fine for LAN listen; **Copy connect / READY / readiness probes** advertise `127.0.0.1` (wildcards are not valid client destinations). Remote peers use this PC’s real LAN IP, not a WSL virtual NIC |

Contrast with products that “helpfully” re-detect a LAN address on update and can land on a WSL/Hyper-V address that is unreachable from WAN/LAN peers. WinServeAI refuses that class of bug by keeping bind config static until the operator edits it.

Prefer **`0.0.0.0` + API key** for shared LAN access instead of pinning one DHCP/WSL address that can change after reboot.

## Restart (manager stop/start, tray Restart)

| Piece | Behavior |
| --- | --- |
| Config | Reloaded from the same YAML path; bind unchanged |
| Child | Previous `llama-server` stopped (CTRL_BREAK → grace → kill) before a new spawn |
| Lockfile | `%LOCALAPPDATA%\WinServeAI\manager.lock` — stale PID cleaned on next acquire |
| GPU | Job Object / stop path releases the process; next start re-detects hardware |

## Power interrupt / crash

| Piece | Behavior |
| --- | --- |
| YAML save | Atomic write (temp + replace) so a cut mid-save should not leave a truncated config |
| Manager kill | Windows Job Object `KILL_ON_JOB_CLOSE` reaps `llama-server` with the manager — no orphan GPU holder on abnormal exit |
| Lockfile | Next launch treats a dead PID as stale and continues |
| Half-written logs | Append-only; rotation closes handles before rename (Windows-safe) |

## Upgrade (Inno Setup)

| Piece | Behavior |
| --- | --- |
| Running app | Setup asks to close in-use binaries (`CloseApplications`) — stop serving before upgrade when possible |
| Config | Not overwritten if present |
| Binaries / pin | `winserve*.exe` and `bin\*` replace with `ignoreversion` |
| Firewall | LAN rule only if that task was selected; uninstall removes the named rule |
| After upgrade | Host/port/model from the previous install still apply; start and confirm Copy connect |

## Reboot

Nothing auto-starts today. After a reboot the API is down until the operator opens the tray/CLI and starts again. That is intentional for MVP (documented in the Windows launch guide).

## Checklist after upgrade or unexpected reboot

1. `winserve doctor` (or tray Start → Ready).
2. Copy connect → confirm bind and bases match what clients use.
3. `.\scripts\smoke-openai.ps1` (or one chat + one Anthropic POST).
4. If LAN clients fail: confirm `server.host` is still what you set (not a WSL IP), firewall task if needed, and API key.

## Fix without reinstall

| Problem | Command |
| --- | --- |
| “What is broken?” | `winserve doctor` / `winserve doctor --fix` |
| Stale lock after crash | `winserve debug --fix-lock` |
| See recent failures | `winserve debug --logs 40 --probe` |
| Wrong model / host / key | `winserve config set … --apply` (reloads resident YAML + starts) |
| Backend wedged | `winserve restart` |
| YAML edited by hand | `winserve stop` → `winserve reload` → `winserve start` |
| Need a newer build | `winserve update` / `winserve update --check` (config kept on upgrade) |
