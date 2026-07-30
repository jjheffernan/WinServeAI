# Project TODO — MVP build

**Source:** [PLAN.md](PLAN.md) · **Maturity:** [readiness/README.md](readiness/README.md) **3.2/5**  
**Branch:** `dev` · **Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md)

Testing / A2 smoke is **non-blocking**. Prefer draft PRs against `dev` that `cargo check`.

## MVP build

Ordered for after-hours / feature-spec drains. One PR per letter-number when possible.

### Now — Resident manager (G)

- [x] **G1** Long-lived `winserve serve` embeds `ServerManager`
- [x] **G2** Lockfile under `%LOCALAPPDATA%\WinServeAI\manager.lock` (PID + pipe); stale if PID dead
- [x] **G3** Named-pipe IPC: `status` / `start` / `stop` / `restart` (+ health/endpoint)
- [x] **G4** CLI `stop` / `restart` / `status` attach to resident owner
- [x] **G5** Single-instance: second serve/tray fails “already running”

### Next — Engine product gaps (H)

- [x] **H1** CREATE_SUSPENDED → assign Job Object → resume
- [x] **H2** Log rotation under `logs/`
- [x] **H3** Optional `/health` readiness fallback (keep `/v1/models` primary)
- [x] **H4** Pin fetch helper + `notices/THIRD_PARTY_NOTICES` stub
- [x] **H5** `scripts/stop.ps1` prefers IPC graceful stop when lockfile exists

### Then — Desktop (E)

- [x] **E1a** Tauri 2 `winserve-tray`; commands → `ServerManager` only
- [x] **E1b** Start / stop / status UI (canonical states only)
- [x] **E1c** Log viewer
- [x] **E1d** Settings → YAML validate/write
- [ ] **E2** Model path picker (local `.gguf` only)
- [ ] **E1e** Quit → `stop()`; Job Object backstop

### Then — Installer (F)

- [ ] **F0** Release layout script (exe + tray + bin + config + notices)
- [ ] **F1** Inno Setup script
- [ ] **F2** Desktop + Start Menu → tray/manager
- [ ] **F3** Firewall only for non-loopback; remove on uninstall
- [ ] **F4** Notices payload (llama.cpp MIT; CUDA notice if needed)
- [ ] **F5** First-run model path guidance

### Ship — Exit polish (I)

- [ ] **I1** Docs: tray + IPC in development / architecture / installer
- [ ] **I2** Readiness re-score + `check_doc_drift.py`
- [ ] **I3** Safe default.yaml + empty model.path guidance
- [ ] **I4** Record shipped `b####` pin in release docs

## Non-blocking / later

| Item | Note |
| --- | --- |
| **A2** Windows smoke | Operator proof; do not gate G–F |
| Unit / CI expansion | Phase 4 or opportunistic |
| NVML FFI | Keep nvidia-smi until needed |
| Phase 4 stability | Crash recovery polish, config migration |
| Phase 5 | Auto-update, service mode, metrics |

## Explicitly do not schedule

* Chat UI, model downloads, multi-backend, Docker, auth, remote management

## See also

- [PLAN.md](./PLAN.md) — full MVP sequence and exit checklist
- [specs/E-desktop.md](./specs/E-desktop.md) · [specs/F-installer.md](./specs/F-installer.md)
- [roadmap.md](./roadmap.md) · [vision.md](./vision.md)
