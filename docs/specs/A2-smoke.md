# A2 — Operator smoke path

**Milestone:** [PLAN.md](../PLAN.md) A2  
**Goal:** On Windows, `winserve start` → `GET /v1/models` **200** → OpenAI-compatible client against `/v1`, with clean stop and no orphan `llama-server`.

This is the Phase 1 exit proof with a **real** pinned binary and GGUF. Unit tests alone do not close A2.

## Automated vs manual

| Step | Automated? | How |
| --- | --- | --- |
| Unit / config / argv | **Yes** (any host) | `scripts/smoke-check.sh` → `cargo test -p winserve`, `print-cmd`, config present |
| Binary present | **Yes** on Windows smoke; **documented** on non-Windows | `smoke-openai.ps1` fails if `bin/llama-server.exe` missing; `smoke-check.sh` notes path |
| Model path exists | **Yes** (Windows smoke) | `smoke-openai.ps1` fails if `model.path` missing on disk |
| Start manager | **Partial** | `-Start` background start, **or** operator runs `cargo run -p winserve -- start` |
| `GET /v1/models` 200 | **Yes** (Windows, server up) | `scripts/smoke-openai.ps1` polls until 200 or timeout |
| `POST /v1/chat/completions` | **Yes** (optional) | Same script when models list non-empty; `-SkipChat` to skip |
| CTRL_BREAK / graceful stop | **Manual** | Ctrl+C on foreground `start` (manager sends CTRL_BREAK to child) |
| Orphan check after manager kill | **Manual** | Task Manager / `Get-Process llama-server` after force-kill of winserve |
| Pin **b9866** download | **Manual** | Operator places binary per [bin/README.md](../../bin/README.md) |

A2 stays **open** in PLAN until an operator has run the full Windows path (including stop + orphan checks). Scripts reduce friction; they do not replace that proof.

## Preconditions (manual setup)

1. **Branch:** `dev`.
2. **Pin:** Download llama.cpp **[b9866](https://github.com/ggml-org/llama.cpp/releases/tag/b9866)** Windows asset (CUDA build preferred when NVIDIA driver matches).
3. Extract into `bin/`:
   - `bin/llama-server.exe`
   - CUDA runtime DLLs **beside** the exe (same directory).
4. Edit `config/default.yaml` (or `WINSERVE_CONFIG`):
   - Set `model.path` to a real `.gguf` on disk.
   - Default bind: `server.host: 127.0.0.1`, `server.port: 8080`.
5. Build manager:

```powershell
cargo build -p winserve --release
# or: cargo run -p winserve -- …
```

## Automated preflight (no GGUF required)

Any host with Rust:

```bash
chmod +x scripts/smoke-check.sh   # once
./scripts/smoke-check.sh
```

Expect: tests pass, `print-cmd` prints resolved `bin/llama-server…` argv, config exists. Does **not** start inference.

## Windows smoke (needs binary + GGUF)

### Option A — operator starts winserve

Terminal 1:

```powershell
cargo run -p winserve -- print-cmd   # confirm argv
cargo run -p winserve -- start
# wait for: READY http://127.0.0.1:8080/v1
```

Terminal 2:

```powershell
.\scripts\smoke-openai.ps1
# or skip chat:
.\scripts\smoke-openai.ps1 -SkipChat
```

### Option B — script starts winserve

```powershell
.\scripts\smoke-openai.ps1 -Start
```

Script exit codes: `0` ok, `1` preconditions, `2` readiness timeout, `3` chat failed, `4` start failed.

On **success**, background `winserve` stays up (script prints PID + `Stop-Process` hint). On **failure**, the script force-stops the process it started so a bad smoke does not leave an orphan manager. Prefer Option A for CTRL_BREAK / Ctrl+C proof.

### curl examples (manual)

```powershell
curl http://127.0.0.1:8080/v1/models

curl http://127.0.0.1:8080/v1/chat/completions `
  -H "Content-Type: application/json" `
  -d "{\"model\":\"local\",\"messages\":[{\"role\":\"user\",\"content\":\"Hello\"}],\"max_tokens\":64}"
```

Use `id` from `/v1/models` as `model` if the client requires an exact match. More clients: [api.md](../api.md).

## Stop (CTRL_BREAK path)

Foreground `winserve start` owns the child. Stop with **Ctrl+C** in that terminal.

Manager stop path (Windows): `CTRL_BREAK` to the process group, grace (~8s), then force kill. Implementation: `app/src/runtime/process.rs`. Details: [backend.md](../backend.md), [research/01-windows-process.md](../research/01-windows-process.md).

Do **not** rely on `winserve stop` for MVP — CLI prints that stop needs a long-running tray/service ([E3](E-desktop.md)).

Emergency (force, no grace):

```powershell
.\scripts\stop.ps1
```

## Orphan check (manual)

After a **normal** Ctrl+C stop:

```powershell
Get-Process -Name "llama-server" -ErrorAction SilentlyContinue
# expect: no process
```

After **force-kill of winserve only** (Task Manager end task / `Stop-Process -Name winserve -Force`), Job Object `KILL_ON_JOB_CLOSE` should reap `llama-server`:

```powershell
Get-Process -Name "winserve","llama-server" -ErrorAction SilentlyContinue
# expect: neither remains
```

If `llama-server` survives, A2 fails — file under process ownership, not smoke scripts.

## Pass criteria (close A2 checkbox)

- [ ] `bin/llama-server.exe` is **b9866** (or documented pin bump)
- [ ] `model.path` points at a real GGUF
- [ ] `winserve start` prints `READY http://…/v1`
- [ ] `.\scripts\smoke-openai.ps1` exits `0` (models + chat)
- [ ] Ctrl+C stop leaves no `llama-server`
- [ ] Force-kill winserve leaves no orphan `llama-server`

## Scripts

| Script | Host | Role |
| --- | --- | --- |
| [`scripts/smoke-check.sh`](../../scripts/smoke-check.sh) | any | tests + `print-cmd` + config; no inference |
| [`scripts/smoke-openai.ps1`](../../scripts/smoke-openai.ps1) | Windows | poll `/v1/models`, optional chat |

## See also

- [PLAN.md](../PLAN.md) — milestone A2
- [api.md](../api.md) — readiness and client examples
- [bin/README.md](../../bin/README.md) — pin **b9866**
- [research/02-llama-readiness.md](../research/02-llama-readiness.md) — probe semantics
- [development.md](../development.md) — first-run steps
