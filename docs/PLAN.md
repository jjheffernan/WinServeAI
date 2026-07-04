# Implementation plan (action items)

**Branch:** `dev` (work) · **`main`** = releases only  
**Maturity:** [readiness/README.md](readiness/README.md) — **2.5/5** (`mvp-partial`)  
**Source backlog:** [TODO.md](TODO.md) · **Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md)

Phase 0 (research + docs architecture) is **done**. This plan is the ordered **code** path to v0.1+.

---

## Milestone A — `winserve start` works (v0.1 engine)

- [ ] **A1.** Pin `llama-server` for local/dev: document exact `b####` in `bin/README.md`; place `bin/llama-server.exe` (binary not committed).
- [ ] **A2.** Smoke path: `cargo run -p winserve -- start` → `GET /v1/models` 200 → OpenAI-compatible client against `/v1`.
- [ ] **A3.** Fail clearly if binary or model path missing (messages already partial — verify and tighten).

**Exit:** Developer on Windows can start API without reading research notes.

---

## Milestone B — Windows process ownership (P0)

- [ ] **B1.** Job Object supervision (`process-wrap` or equivalent) so force-kill of manager reaps `llama-server` ([research/01](research/01-windows-process.md)).
- [ ] **B2.** Real graceful stop: `CREATE_NEW_PROCESS_GROUP` + `CTRL_BREAK_EVENT` (replace no-op `send_ctrl_break`).
- [ ] **B3.** Port preflight / sleep-wake reclaim before spawn ([llama.cpp #20648](https://github.com/ggml-org/llama.cpp/discussions/20648)).
- [ ] **B4.** Exit watcher → `Status::Crashed` when child dies unexpectedly.

**Exit:** `runtime-process` readiness score ≥ **3.5**; no orphan `llama-server` after manager death.

---

## Milestone C — Hardware defaults (P0)

- [ ] **C1.** DXGI adapter + VRAM inventory (`app/src/system/gpu.rs`).
- [ ] **C2.** Optional NVML device-wide free/total (not per-process under WDDM).
- [ ] **C3.** CPU-only path when no GPU (`-ngl 0`); with GPU prefer `--fit on` ([research/03](research/03-hardware-detection.md)).
- [ ] **C4.** Confirm `runtime/llama.rs` auto path exercises `--fit` when GPUs present.

**Exit:** `system` readiness score ≥ **3.0**; auto GPU no longer always CPU-only.

---

## Milestone D — Tests & polish (Phase 1 exit)

- [ ] **D1.** Unit tests: config load/validate, argv snapshots from YAML.
- [ ] **D2.** Health timeout / readiness behavior tests (mock HTTP or integration).
- [ ] **D3.** Process stub exe test on Windows CI (spawn/stop).
- [ ] **D4.** Log line timestamps in `server-logs`.
- [ ] **D5.** Stronger config validation (port range, model path warning).
- [ ] **D6.** Refresh readiness scorecards + operator **Readiness** blocks ([policies/doc-drift.md](policies/doc-drift.md)).

**Exit:** Phase 1 deliverable met — launch, Ready, clean shutdown through `ServerManager`; checklist in [roadmap.md](roadmap.md) v0.1–v0.2.

---

## Milestone E — Desktop (Phase 2)

- [ ] **E1.** Thin UI (Tauri or chosen) — start / stop / status / logs / settings only via `ServerManager`.
- [ ] **E2.** Model **path** picker (no downloads).
- [ ] **E3.** Long-running manager so CLI/UI `stop`/`restart` work (lockfile+IPC or tray-owned process).

**Exit:** No CLI required for basic use.

---

## Milestone F — Installer (Phase 3)

- [ ] **F1.** Inno Setup script: `winserve.exe`, `bin/llama-server.exe`, `config/`, notices.
- [ ] **F2.** Desktop + Start Menu shortcut → manager.
- [ ] **F3.** Firewall rule only when bind is non-loopback; delete on uninstall.
- [ ] **F4.** `THIRD_PARTY_NOTICES` (llama.cpp MIT) in install payload ([research/04](research/04-installer-licensing.md)).

**Exit:** Install → Start → API available.

---

## Explicitly out of scope (do not schedule)

* Backend traits / plugins / multi-provider
* Chat UI, model marketplace, HuggingFace downloads
* Docker, auth, remote management (Phase 5+ only if ever)

---

## Doc maintenance (ongoing)

- [ ] After material code changes: run `python3 scripts/check_doc_drift.py` (see [policies/doc-drift.md](policies/doc-drift.md)).
- [ ] Re-score modules per [readiness/PLAN.md](readiness/PLAN.md); update `readiness/*.md` and operator guide banners.
- [ ] Keep citations current in research notes when upstream llama.cpp behavior changes.

## See also

- [TODO.md](./TODO.md) — priority backlog from readiness scores
- [roadmap.md](./roadmap.md) — phased product plan
- [readiness/README.md](./readiness/README.md) — maturity dashboard
- [architecture.md](./architecture.md) — appliance rules
- [policies/doc-drift.md](./policies/doc-drift.md) — score and claim consistency
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
