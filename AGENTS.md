# Agent Instructions

WinServeAI is a **Windows-native llama.cpp appliance wrapper**.

```text
App/CLI → ServerManager → runtime (llama + process) → bin/llama-server.exe → /v1
```

## Do not mess this up

1. **One backend only** — llama.cpp. No backend traits, plugins, or multi-provider layers.
2. **One orchestrator** — `ServerManager` (`app/src/server/manager.rs`). UI/CLI only call it.
3. **Raw llama flags only in** `app/src/runtime/llama.rs`.
4. **YAML config is source of truth** (`config/default.yaml`).
5. **External boundary is explicit** — `bin/llama-server.exe` is not our code.
6. **No** model downloads, chat UI, Docker, agent frameworks, or dashboards.

Read `docs/architecture.md` and `docs/build-spec.md` before structural changes.

## Branches

| Branch | Purpose |
| --- | --- |
| **`dev`** | Default working branch — experimental / preview. Land work here. |
| **`main`** | Releases only. Do not commit day-to-day changes on `main`. |

Open PRs against **`dev`**. Release promotions are PRs from `dev` → `main`.

## Layout

| Path | Role |
| --- | --- |
| `app/src/server/` | Manager, config, health, logs |
| `app/src/runtime/` | Process + llama argv |
| `app/src/system/` | GPU / memory / network probes |
| `app/src/api/` | OpenAI URL helpers (passthrough) |
| `bin/` | External llama-server binary |
| `config/` | default.yaml |
| `logs/` | server.log, llama.log, error.log |

## Skills

Canonical skills: `.agents/skills/` (agent-agnostic).

| Skill | Use when |
| --- | --- |
| `pr-worktree-review` | PR worktree review loop |
| `grill-me` / `ponytail` / `caveman` | Planning / minimalism / brevity |

```bash
python3 -m scripts.pr_review_loop --config examples/pr_review_loop.json --dry-run --no-push
```
