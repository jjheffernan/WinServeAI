# Pair-QA — A2 smoke + E/F specs

**Role:** partner / QA (pair-programming support)  
**Branch:** `dev`  
**Date:** 2026-07-04  
**Scope:** A2 operator smoke scripts/spec; Phase 2 desktop (`E-desktop`) and Phase 3 installer (`F-installer`) specs  
**Readiness:** not rescored here (leftovers owns scores)

---

## Deliverables reviewed

| Deliverable | Path | Verdict |
| --- | --- | --- |
| A2 smoke spec | `docs/specs/A2-smoke.md` | **Pass** (minor notes) |
| OpenAI smoke script | `scripts/smoke-openai.ps1` | **Pass** after small QA patch |
| Preflight script | `scripts/smoke-check.sh` | **Pass** |
| Desktop spec | `docs/specs/E-desktop.md` | **Pass** — architecture-aligned |
| Installer spec | `docs/specs/F-installer.md` | **Pass** — architecture-aligned |

---

## Automated checks

| Check | Result |
| --- | --- |
| `cargo test -p winserve` | **14 passed** |
| `python3 scripts/check_doc_drift.py` | **doc drift OK** |
| `./scripts/smoke-check.sh` | **Pass** (tests + `print-cmd` + config; notes missing binary on this host) |
| `pwsh` parse of `smoke-openai.ps1` | **Skipped** — no PowerShell on this macOS agent host; logic reviewed by hand |
| Readiness scores / banners | Leftovers owns rescore (operator-path / specs only; A2 checkbox stays open). |

---

## 1. A2 smoke — findings

### What implementer got right

* Pin **b9866** called out in preconditions, fail messages, and pass criteria.
* Primary probe is **`GET /v1/models`** (not `/health` alone); default timeout **120s** matches `health.rs` / manager.
* Host/port read from YAML (`WINSERVE_CONFIG` / `-ConfigPath`); defaults `127.0.0.1:8080`.
* Chat uses `data[0].id` from models list (not a hardcoded name).
* Documents MVP: stop via **Ctrl+C**, not `winserve stop` (E3).
* Manual orphan check after force-kill of manager (Job Object).
* Clear automated vs manual table; A2 stays open until operator proof.
* `smoke-check.sh` is safe for CI/macOS (no GGUF required).

### Gaps / risks (residual)

| Item | Severity | Notes |
| --- | --- | --- |
| **No binary version assert** | Low | Script cannot prove the exe is b9866 (no `--version` gate). Operator checklist is the control. |
| **`-Start` uses Hidden window** | Low | CTRL_BREAK / Ctrl+C proof needs **Option A** (foreground). Spec now states this. |
| **`scripts/stop.ps1` global kill** | Low | Spec lists it as emergency only — good. Still kills *all* `winserve`/`llama-server` by name. |
| **YAML scalar reader** | Low | Minimal regex; fine for `default.yaml`. Breaks if operators use multi-doc YAML or unusual quoting. |
| **503 handling** | OK | Relies on `Invoke-WebRequest` throwing on 5xx and continuing the poll loop — correct on Windows PowerShell. |
| **Full A2 not run here** | Expected | Needs Windows + b9866 + GGUF. |

### QA patches applied

1. **`scripts/smoke-openai.ps1`:** on failure after `-Start`, stop the background manager (avoid orphan from failed smoke); on success, print PID + `Stop-Process` hint; mention pin **b9866** at start.
2. **`docs/specs/A2-smoke.md`:** document Option B lifecycle (success leaves process up; failure cleans up; prefer Option A for CTRL_BREAK).

---

## 2. E-desktop vs architecture

**Match: yes.**

| Requirement | Spec coverage |
| --- | --- |
| ServerManager-only UI | Explicit diagram + API table; webview never owns readiness/spawn |
| No downloads / chat | Non-goals + acceptance #5, #8 |
| No llama flags in UI | Non-goals + “do not touch `runtime/llama.rs` from UI” |
| Model **path** picker only | E2 |
| Long-running manager (E3) | Options A–C; **recommends C** (tray + lockfile + named pipe) — sound |
| Ready ≠ process alive | UI rules #2; state machine cites research/05 |
| No Tauri sidecar for inference | Non-goals |

No red flags. Implementation order (IPC/CLI attach before Tauri chrome) is sensible.

**Note for implementers:** Option C IPC is Windows-first; keep Job Object ownership in the tray/manager process, not in CLI clients.

---

## 3. F-installer vs architecture

**Match: yes.** Explicitly prefers `docs/installer.md` layout over research/04’s older `{app}\bin\winserve.exe` sketch.

| Requirement | Spec coverage |
| --- | --- |
| Payload: manager + `bin/llama-server.exe` + config + notices | F1 |
| Shortcut → manager/tray, not backend | F2 |
| Firewall only non-loopback; program rule; uninstall delete | F3 |
| MIT notices in install tree | F4 + ollama#3185 |
| No models / no chat / no downloaders | Non-goals + acceptance |
| CUDA DLLs beside exe; Attachment A | CUDA section |
| Pin at release time | F1 Pin (b#### from release docs / bin/README) |

Minor: pin text is generic `b####` (correct for release packaging); current dev pin remains **b9866** in `bin/README.md`.

---

## 4. Test plan — A2 on Windows (manual)

### Setup

1. Checkout `dev`; `cargo build -p winserve --release`.
2. Download llama.cpp **[b9866](https://github.com/ggml-org/llama.cpp/releases/tag/b9866)** Windows asset.
3. Extract `llama-server.exe` (+ CUDA DLLs) into `bin\`.
4. Set `model.path` in `config\default.yaml` to a real `.gguf`.
5. Optional: `cargo run -p winserve -- print-cmd`.

### Preflight (any host)

```bash
./scripts/smoke-check.sh
```

### Happy path (Option A — preferred for stop proof)

6. Terminal A: `cargo run -p winserve -- start` → wait for `READY http://127.0.0.1:8080/v1`.
7. Terminal B: `.\scripts\smoke-openai.ps1` → exit `0`.
8. Terminal A: **Ctrl+C** → `Get-Process llama-server` empty.

### Orphan (Job Object)

9. Start again; force-kill **winserve only**.
10. Assert no `llama-server` remains.

### Option B (script start)

```powershell
.\scripts\smoke-openai.ps1 -Start
# on success: Stop-Process -Id <pid> when done
```

### Pass criteria (close A2)

- [ ] Binary is **b9866**
- [ ] READY + smoke script exit 0
- [ ] Ctrl+C: no orphan
- [ ] Force-kill manager: no orphan

---

## 5. Drift / leftovers

* **A2 remains open** until Windows operator proof (GGUF + **b9866** + stop/orphan checks). Scripts/spec do not close the checkbox.
* Leftovers rescored operator-path / specs only: `bin` **2.2**, `docs` **4.0**, `scripts-ops` **3.2**, `installer` **1.6** (docs dim; implementation still 0). Overall maturity **3.1/5**. `ui` unchanged stub.
* Leftovers script touch: port-busy fail on `smoke-openai.ps1 -Start`; pin/stop notes in script headers.
* Pre-existing research/04 vs installer.md layout drift is acknowledged in F-installer (correct preference). research/04 align later.

---

## 6. Files modified by pair-QA

| File | Action |
| --- | --- |
| `docs/audit/pair-qa-a2-ef.md` | Created (this file) |
| `scripts/smoke-openai.ps1` | Minimal: fail cleanup, success stop hint, pin note on `-Start` |
| `docs/specs/A2-smoke.md` | Option B lifecycle note |

No readiness scores, Tauri, or Inno implementation.

---

## See also

- [PLAN.md](../PLAN.md) · [architecture.md](../architecture.md) · [prior-art.md](../prior-art.md)
- [installer.md](../installer.md) · [research/01–05](../research/05-server-manager.md)
- Specs: [A2-smoke.md](../specs/A2-smoke.md) · [E-desktop.md](../specs/E-desktop.md) · [F-installer.md](../specs/F-installer.md)
