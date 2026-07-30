# Implementation plan — full MVP build-out

**Branch:** `dev` (work) · **`main`** = releases only  
**Maturity:** [readiness/README.md](readiness/README.md) — **3.4/5** (`mvp-partial`)  
**Source backlog:** [TODO.md](TODO.md) · **Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md)  
**Vision MVP:** [vision.md](vision.md) · **Build success:** [build-spec.md](build-spec.md)

---

## MVP definition (done when)

A Windows user can:

1. Run the installer (or a packaged layout matching it)
2. Launch the tray / manager from a shortcut
3. Pick a local GGUF path and start
4. Hit `http://127.0.0.1:8080/v1` from an OpenAI-compatible client
5. Stop / quit and free the GPU (no orphan `llama-server`)

Matches roadmap **v0.4** + vision MVP. Phase 4 stability (migration, extensive hardware matrix) is **post-MVP**.

### Policy for this build-out

* **Testing is non-blocking.** Ship slices without waiting on A2 Windows smoke, new unit suites, or CI green as a gate. Prefer draft PRs that compile (`cargo check -p winserve`) and document manual verify steps.
* **Architecture hard rules stay.** One backend (llama.cpp), one orchestrator (`ServerManager`), raw flags only in `runtime/llama.rs`, YAML source of truth. No chat UI, downloads, Docker, multi-backend.
* **PR size.** One milestone letter slice per PR when possible (e.g. `E3a` lockfile+IPC, then `E1` shell). Prefer draft PRs against `dev`.

---

## Current state (already done)

| Area | Status |
| --- | --- |
| Phase 0 research + appliance ADR | Done |
| **A1/A3** pin + fail-hard missing binary/model | Done |
| **B** Job Object + CTRL_BREAK + port preflight + crash status | Done |
| **C** DXGI / nvidia-smi / `--fit` vs `-ngl 0` | Done |
| **D** unit tests + validation + readiness docs | Done |
| **A2** operator smoke on Windows | Open — **non-blocking** |
| **E** desktop + resident manager | Done on `dev` (E1a–E1e, E2); tray wraps ServerManager + lockfile/IPC |
| **F** Inno installer | Done on `dev` (F0–F5); ISCC compile + install→Ready proof still open ([specs/F-installer.md](specs/F-installer.md)) |

---

## Build sequence (ordered)

Work top → bottom. Later slices assume earlier APIs exist.

### Milestone G — Resident manager + CLI attach (foundation)

**Why first:** Unlocks stop/restart for CLI and becomes the process that owns Job Object for tray/UI. Spec Option C in [E-desktop.md](specs/E-desktop.md).

| ID | Slice | Deliverable |
| --- | --- | --- |
| **G1** | `winserve serve` (or tray-headless mode) | Long-lived process embeds `ServerManager`; foreground `start` can remain for one-shot dev |
| **G2** | Lockfile | `%LOCALAPPDATA%\WinServeAI\manager.lock` (PID + pipe name); stale lock if PID dead |
| **G3** | Named-pipe IPC | Minimal cmds: `status`, `start`, `stop`, `restart`, `health` / endpoint; localhost same-user only |
| **G4** | CLI attach | `winserve status\|stop\|restart` talk to resident owner; clear error if none running |
| **G5** | Single-instance | Second `serve`/tray start refuses with “already running” |

**Exit:** With serve running, CLI can stop/restart without owning the child. Job Object lifetime stays in the owner process.

**Files (expected):** `app/src/ipc/` (new), `app/src/main.rs`, `app/src/server/manager.rs`, docs for ops.

---

### Milestone H — Engine gaps that matter for MVP (product, not tests)

Skip pure test-only work. Do only what users feel.

| ID | Slice | Deliverable |
| --- | --- | --- |
| **H1** | Job assign race | CREATE_SUSPENDED → assign job → resume (readiness gap on `runtime-process`) |
| **H2** | Log rotation | Size/age rotate under `logs/` so overnight runs don’t fill disk |
| **H3** | `/health` fallback | If pin exposes it, use as alternate readiness; keep `/v1/models` primary |
| **H4** | Binary pin helpers | Script or doc to fetch pinned `b####` into `bin/`; ship `THIRD_PARTY_NOTICES` stub under `notices/` |
| **H5** | Ops scripts | Prefer IPC graceful stop in `scripts/stop.ps1` when lockfile/pipe exists; force remains fallback |

**Defer:** NVML FFI, Windows CI matrix, unit-test expansion, A2 checkbox.

---

### Milestone E — Desktop shell (Phase 2)

Spec: [specs/E-desktop.md](specs/E-desktop.md). UI only calls `ServerManager` (in-process in tray) or the same IPC surface.

| ID | Slice | Deliverable |
| --- | --- | --- |
| **E3** | *(covered by G)* | Tray **is** the G owner; embed manager + Job Object |
| **E1a** | Tauri 2 scaffold | `winserve-tray` binary; Rust commands wrap manager only — no spawn of `llama-server` from webview |
| **E1b** | Start / stop / status | Badge matches states: Stopped, Starting, Ready, Failed, Stopping, Crashed (no fake “Running” = Ready) |
| **E1c** | Log viewer | Tail/subscribe `logs/` (or manager forward); timestamps; viewer only |
| **E1d** | Settings | Edit YAML fields via config load/validate/write; reject unsafe edits while Starting/Ready/Stopping |
| **E2** | Model path picker | Native dialog → `.gguf` path → persist `model.path`; no downloads |
| **E1e** | Quit path | Quit → `stop()`; Job Object backstop |

**Exit:** No terminal required for basic use. Copy-paste OpenAI base URL visible.

**Non-goals in E:** chat, downloads, hot-swap model while Ready, second orchestrator.

---

### Milestone F — Installer (Phase 3)

Spec: [specs/F-installer.md](specs/F-installer.md). Vision requires this for MVP.

| ID | Slice | Deliverable |
| --- | --- | --- |
| **F0** | Release layout script | Produce `{app}` tree: `winserve.exe`, `winserve-tray.exe`, `bin/llama-server.exe` (+ CUDA DLLs if applicable), `config/default.yaml`, `notices/` |
| **F1** | Inno Setup script | Install that tree under Program Files (or chosen dir) |
| **F2** | Shortcuts | Desktop + Start Menu → **tray/manager**, never `llama-server` alone |
| **F3** | Firewall | Add rule only when bind is non-loopback; delete on uninstall |
| **F4** | Notices | `THIRD_PARTY_NOTICES` + llama.cpp MIT (+ NVIDIA notice for CUDA builds) |
| **F5** | First-run | Minimal: set `model.path` (wizard or “open settings on first launch”); no weight shipping |

**Exit:** Install → Start → API available on a clean Windows machine with a user-supplied GGUF.

---

### Milestone I — MVP exit polish (thin)

| ID | Slice | Deliverable |
| --- | --- | --- |
| **I1** | Operator docs | Update `docs/development.md`, `docs/installer.md`, architecture for tray + IPC |
| **I2** | Readiness refresh | Re-score modules + banners; `python3 scripts/check_doc_drift.py` |
| **I3** | Default config | Safe localhost defaults; empty/placeholder `model.path` with clear first-run guidance |
| **I4** | Pin record | Release docs record exact `b####` shipped in installer |

**MVP ship checklist**

- [x] Resident manager + CLI attach (G) — implemented on `dev`
- [x] Tray: start/stop/status/logs/settings + path picker (E) — implemented on `dev`
- [x] Installer packages tray + pinned binary + notices (F) — sources + staging on `dev`; ISCC/install proof open
- [ ] Quit/stop leaves no orphan `llama-server` — code present; Windows operator proof open (A2)
- [ ] OpenAI client works against `/v1` with a local GGUF — A2 open
- [x] No chat UI / download / multi-backend surface in the product

---

## Explicitly out of scope (do not schedule for MVP)

* Backend traits / plugins / multi-provider
* Chat UI, model marketplace, HuggingFace downloads
* Docker, auth, remote management, metrics dashboard
* Auto-updater, Windows service mode (Phase 5+)
* Config schema migration framework (Phase 4)
* Blocking on A2 smoke, full Windows CI, or broad unit-test coverage

---

## After-hours / overnight queueing

Skills: `.agents/skills/after-hours/` ([heff-skills](https://github.com/jjheffernan/heff-skills)). Config: `.cursor/after-hours-loop.config.json`.

Suggested night Sources (paste into `/after-hours`):

```text
Sources:
  - todo-md: path docs/TODO.md section "MVP build"
  - feature-spec: docs/PLAN.md section "Build sequence"
maxPrs: 2
priority: todo-first
```

Preferred overnight order: **G1→G5 → H1 → E1a→E2 → F0→F5 → I***. Skip A2 / pure tests unless explicitly in Sources.

---

## Historical Phase 1 checklist (complete)

### Milestone A — `winserve start` works (v0.1 engine)

- [x] **A1.** Pin `llama-server` for local/dev: document exact `b####` in `bin/README.md`
- [ ] **A2.** Operator smoke (non-blocking for MVP build-out)
- [x] **A3.** Fail clearly if binary or model path missing

### Milestone B — Windows process ownership

- [x] **B1–B4.** Job Object, CTRL_BREAK, port preflight, crash status

### Milestone C — Hardware defaults

- [x] **C1–C4.** DXGI, nvidia-smi, `--fit` / `-ngl 0`

### Milestone D — Phase 1 polish

- [x] **D1–D6.** Tests, timestamps, validation, readiness docs

---

## Doc maintenance (ongoing)

- After material code changes: `python3 scripts/check_doc_drift.py`
- Re-score modules per [readiness/PLAN.md](readiness/PLAN.md) when a milestone exits
- Keep research citations current when upstream llama.cpp behavior changes

## See also

- [TODO.md](./TODO.md) — priority backlog for agents / after-hours
- [roadmap.md](./roadmap.md) — phased product plan
- [specs/E-desktop.md](./specs/E-desktop.md) · [specs/F-installer.md](./specs/F-installer.md)
- [architecture.md](./architecture.md) · [vision.md](./vision.md) · [build-spec.md](./build-spec.md)
- [policies/doc-drift.md](./policies/doc-drift.md)
