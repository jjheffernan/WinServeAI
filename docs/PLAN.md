# Implementation plan (action items)

**Branch:** `dev` (work) · **`main`** = releases only  
**Maturity:** [readiness/README.md](readiness/README.md) — **3.0/5** (`mvp-partial`)  
**Source backlog:** [TODO.md](TODO.md) · **Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md)

Phase 0 (research + docs architecture) is **done**. Phase 1 engine code (A–D except operator smoke **A2**) is **done**; readiness refresh (**D6**) is **done**.

---

## Milestone A — `winserve start` works (v0.1 engine)

- [x] **A1.** Pin `llama-server` for local/dev: document exact `b####` in `bin/README.md`; place `bin/llama-server.exe` (binary not committed).
- [ ] **A2.** Smoke path: `cargo run -p winserve -- start` → `GET /v1/models` 200 → OpenAI-compatible client against `/v1`. *(operator: needs Windows + binary + GGUF)*
- [x] **A3.** Fail clearly if binary or model path missing (messages already partial — verify and tighten).

**Exit:** Developer on Windows can start API without reading research notes.

---

## Milestone B — Windows process ownership (P0)

- [x] **B1.** Job Object supervision (`KILL_ON_JOB_CLOSE` via `windows` crate) so force-kill of manager reaps `llama-server` ([research/01](research/01-windows-process.md)).
- [x] **B2.** Real graceful stop: `CREATE_NEW_PROCESS_GROUP` + `CTRL_BREAK_EVENT` (replace no-op `send_ctrl_break`).
- [x] **B3.** Port preflight fails start when port unavailable ([llama.cpp #20648](https://github.com/ggml-org/llama.cpp/discussions/20648)).
- [x] **B4.** Exit watcher → `Status::Crashed` when child dies unexpectedly (`get_status`).

**Exit:** `runtime-process` readiness score ≥ **3.5**; no orphan `llama-server` after manager death. *(met: **3.8**)*

---

## Milestone C — Hardware defaults (P0)

- [x] **C1.** DXGI adapter + VRAM inventory (`app/src/system/gpu.rs`) on Windows.
- [x] **C2.** Optional device-wide totals via `nvidia-smi` (NVML FFI deferred).
- [x] **C3.** CPU-only path when no GPU (`-ngl 0`); with GPU prefer `--fit on` ([research/03](research/03-hardware-detection.md)).
- [x] **C4.** Confirm `runtime/llama.rs` auto path exercises `--fit` when GPUs present (unit tests).

**Exit:** `system` readiness score ≥ **3.0**; auto GPU no longer always CPU-only. *(met: **3.6**)*

---

## Milestone D — Tests & polish (Phase 1 exit)

- [x] **D1.** Unit tests: config load/validate, argv snapshots from YAML.
- [x] **D2.** Health timeout / readiness behavior tests (timeout when nothing listens).
- [x] **D3.** Process stub command test (spawn/stop `sleep` / `ping`).
- [x] **D4.** Log line timestamps in `server-logs`.
- [x] **D5.** Stronger config validation (layers, context, host; model path warnings).
- [x] **D6.** Refresh readiness scorecards + operator **Readiness** blocks ([policies/doc-drift.md](policies/doc-drift.md)).

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

## Next steps

1. **A2** — Operator smoke on Windows: place `bin/llama-server.exe` (**b9866**), set `model.path`, `cargo run -p winserve -- start` → `GET /v1/models` 200 → OpenAI client against `/v1`. Confirm CTRL_BREAK stop and no orphan after manager kill.
2. **E** — Desktop shell (start/stop/status/logs/settings via `ServerManager` only) + model path picker; **E3** long-running manager so CLI/UI `stop`/`restart` attach.
3. **F** — Inno Setup installer: ship `winserve.exe` + pinned `llama-server.exe` + config + notices; shortcut; firewall only for non-loopback.

---

## Explicitly out of scope (do not schedule)

* Backend traits / plugins / multi-provider
* Chat UI, model marketplace, HuggingFace downloads
* Docker, auth, remote management (Phase 5+ only if ever)

---

## Doc maintenance (ongoing)

- [x] After material code changes: run `python3 scripts/check_doc_drift.py` (see [policies/doc-drift.md](policies/doc-drift.md)). *(D6 pass)*
- [x] Re-score modules per [readiness/PLAN.md](readiness/PLAN.md); update `readiness/*.md` and operator guide banners. *(D6 pass)*
- [ ] Keep citations current in research notes when upstream llama.cpp behavior changes.

## See also

- [TODO.md](./TODO.md) — priority backlog from readiness scores
- [roadmap.md](./roadmap.md) — phased product plan
- [readiness/README.md](./readiness/README.md) — maturity dashboard
- [architecture.md](./architecture.md) — appliance rules
- [policies/doc-drift.md](./policies/doc-drift.md) — score and claim consistency
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
