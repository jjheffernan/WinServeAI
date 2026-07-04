# Apply-worktree audit

**Date:** 2026-07-04  
**Range:** `1f8522c..b5f36e3` (fast-forward onto `main`)  
**Skills used:** `caveman-review`, `ponytail-review`, `pr-worktree-review` (contracts), module readiness rubric  

**Apply result:** `refactor/minimal-appliance` merged to `main`. Research + dry-run worktrees removed. `cargo check -p winserve` **PASS**.

---

## Caveman-review (correctness / risk)

One line per finding: location, problem, fix.

- `app/src/runtime/process.rs:L116-122`: 🟡 risk: `send_ctrl_break` is no-op. Implement `GenerateConsoleCtrlEvent` / Job Object or document force-kill-only as accepted MVP.
- `app/src/main.rs` `stop`/`restart`: 🟡 risk: CLI claims stop/restart but exits failure. Wire long-running manager or remove commands until tray/service exists.
- `app/src/system/gpu.rs`: 🟡 risk: empty GPU list forces CPU `--n-gpu-layers 0` path always. Implement DXGI or detect via `nvidia-smi` fallback before shipping auto GPU.
- `app/src/server/health.rs`: 🔵 nit: only polls `/v1/models`. Optional `/health` fallback already documented; add when pin supports it.
- `app/src/server/logs.rs`: 🔵 nit: no timestamps on lines. Prefix ISO time for crash forensics.
- `scripts/stop.ps1`: 🟡 risk: `Stop-Process -Force` orphans no Job Object children today but will fight graceful stop later. Prefer signaling manager when it exists.
- `bin/`: 🔴 bug (release): no `llama-server.exe` — `start` always fails until operator drops binary. Ship pin instructions in first-run / installer only is OK for dev; block release checklist.
- Tests: 🔴 bug (maturity): zero automated tests. Add config parse + argv snapshot tests before Phase 1 exit.

**Looks good:** single `ServerManager` owner; readiness waits on HTTP; config has no raw flags; docs/readiness scores honest.

---

## Ponytail-review (over-engineering)

One line per finding: location, what to cut, replacement.

- `app/src/api/openai.rs`: yagni: thin URL helpers only — keep (not a proxy). No cut.
- `docs/research/doc-build/*.research.md`: shrink: research briefs served rebuild; keep for audit trail, do not expand.
- `scripts/pr_review_loop/hooks/echo_*.py`: yagni: stubs for dry-run — keep until real agent cmds wired.
- No `packages/*` backend trait reintroduced — **good** (prior monorepo was correctly deleted).
- `app/ui/.gitkeep`: delete: empty Phase 2 path is fine; no code to cut.

**No speculative multi-backend layers found.** Appliance shape holds.

---

## PR-worktree-review contract check

| Rule | Status |
| --- | --- |
| Never modify base during loop work | PASS (loop only used dry-run; apply was explicit merge) |
| UI → ServerManager → runtime → bin | PASS |
| No backend traits | PASS |
| Feature branch only for loop | PASS (`feature/example-a` dry-run removed) |

---

## Subagent scores

Rubric (0–5): **Deliverable** · **Accuracy** · **Autonomy** (no stall/interrupt) · **Appliance alignment** · **Overall**

| Subagent / swarm | Deliverable | Accuracy | Autonomy | Alignment | Overall | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Prior-art research | 5 | 4 | 4 | 3 | **4.0** | Strong links; initially multi-backend lean, later merged |
| Phase-0 research (01–05) | 5 | 4 | 5 | 3 | **4.3** | Deep and useful; path drift needed cleanup pass |
| Doc-build file agents (×11) | 5 | 5 | 4 | 5 | **4.8** | Operator guides complete; transcript lag only |
| Doc orchestrator | 4 | 5 | 2 | 5 | **4.0** | STATUS excellent; stalled until interrupt/resume |
| Path-drift cleanup | 5 | 5 | 3 | 5 | **4.5** | Banners + readiness wording; transcript incomplete |
| Module readiness scorer | 5 | 5 | 4 | 5 | **4.8** | Dashboard + pipes; honest scores |
| **Swarm average** | | | | | **4.4** | |

### Autonomy incidents

| Agent | Issue | Resolution |
| --- | --- | --- |
| Doc orchestrator | Stuck after first tool batch | Interrupt + resume with sibling-complete prompt |
| Path-drift cleanup | Transcript frozen at 2 lines | Files still written; no interrupt needed |
| Readiness scorer | Transcript lag | Deliverables complete without interrupt |

---

## Verdict

| Check | Result |
| --- | --- |
| Worktrees applied to `main` | **PASS** (FF merge) |
| Worktrees removed | **PASS** |
| Build | **PASS** (`cargo check -p winserve`) |
| Architecture regression | **PASS** (no packages monorepo) |
| Ready for Phase 1 coding | **YES** — follow [TODO.md](../TODO.md) and [readiness/README.md](../readiness/README.md) |
| Ready for release | **NO** — bin pin, Windows stop, hardware, tests, installer |

**Main is ahead of `origin/main` by 10 commits** (not pushed).
