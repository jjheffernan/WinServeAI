# Documentation build status

**Worktree:** `research-06cce628` · **Branch:** `refactor/minimal-appliance`  
**Polled:** 2026-07-04 · Orchestrator pass after sibling builders finished.

Architecture lock: appliance wrapper — `ServerManager` only, llama.cpp only, no backend traits.

## Spot-check (rebuilt operator docs)

| Check | Result |
| --- | --- |
| Affirmative `packages/*` layout | **PASS** — none in rebuilt guides |
| “Backend interfaces” | **PASS** — none |
| `packages/*` only as ban/warning | OK in `development.md`, `release-process.md` |
| Phase-0 research `01–05` / `prior-art.md` | **PASS** — path-note banners + light `app/src/…` mapping (see leftover cleanup) |

## Per-file status

Legend: **PASS** = present and appliance-aligned · **GAP** = missing or needs follow-up.

| Target | Research | Build | Verdict | Quality notes |
| --- | --- | --- | --- | --- |
| `api.md` | `api.research.md` PASS | PASS (~148 lines) | **PASS** | Passthrough `/v1`, readiness, no `packages/api` |
| `backend.md` | `backend.research.md` PASS | PASS (~147 lines) | **PASS** | Runtime framing; flags only in `runtime/llama.rs` |
| `configuration.md` | `configuration.research.md` PASS | PASS (~198 lines) | **PASS** | Full schema + localhost/LAN; no raw flags |
| `contributing.md` | `contributing.research.md` PASS | PASS (~59 lines) | **PASS** | ServerManager rules; no “Backend interfaces” |
| `development.md` | `development.research.md` PASS | PASS (~139 lines) | **PASS** | Single-crate layout; explicit no-`packages/` rule |
| `installer.md` | `installer.research.md` PASS | PASS (~101 lines) | **PASS** | Inno-first; firewall; links research/04 |
| `logging.md` | `logging.research.md` PASS | PASS (~66 lines) | **PASS** | Unified `logs/`; capture requirements |
| `release-process.md` | `release-process.research.md` PASS | PASS (~125 lines) | **PASS** | `winserve` crate only; pin policy |
| `research.md` | `research.research.md` PASS | PASS (~69 lines) | **PASS** | Checklist updated; no `packages/config` |
| `roadmap.md` | `roadmap.research.md` PASS | PASS (~99 lines) | **PASS** | No backend-abstraction / plugin track |
| `adr/README.md` | `adr.research.md` PASS | PASS | **PASS*** | Index exists; needs ADR 0001 listed (see gaps) |
| `adr/0001-appliance-architecture.md` | (via adr.research) | PASS (~26 lines) | **PASS** | Accepted appliance lock |

\*README still said “No numbered ADRs yet” at poll time — orchestrator fixes in same pass.

### Already solid (not rebuild targets)

| File | Status |
| --- | --- |
| `architecture.md` | canonical — not rewritten |
| `vision.md` | canonical — not rewritten |
| `build-spec.md` | canonical |
| `prior-art.md` | canonical research dump |
| `research/01–05-*.md` | canonical phase-0 deep dives |
| `INDEX.md` | present (orchestrator) |
| `PLAN.md` | present |

### Doc-build research briefs

All PLAN targets have `*.research.md`:

`api`, `backend`, `configuration`, `contributing`, `development`, `installer`, `logging`, `release-process`, `research`, `roadmap`, `adr`.

## Remaining gaps

1. **Open research checklist items** — migration strategy, rotating logs, metrics endpoint, UI topics (intentional; not blocking doc-build).
2. **ADR backlog** — 0001 notes pin policy, Windows stop semantics, and installer choice as future ADRs (not required for this pass).

## Leftover cleanup (follow-up)

Completed after operator guides (11/11 PASS):

1. **Path drift** — Added appliance path-note banners to `research/01–05` and `prior-art.md`. Light in-body fixes: broken `../../packages/…` links → `app/src/…`; recommendation headings map to `app/src/server`, `app/src/runtime`, `app/src/system`, `bin/`. Scaffolding P0/P1 in `prior-art.md` no longer prescribe `packages/*` or a Rust `Backend` trait. Pin references use `bin/llama-server.exe` + release-doc version tags (not `vendor/llama.cpp`).
2. **Readiness consistency** — Primary probe documented as **`GET /v1/models`** (`app/src/server/health.rs`). `GET /health` (503 loading / 200 ready) is an optional alternate when present on the pin — not the only probe. Updated in `research/02-llama-readiness.md`, `prior-art.md`, and light touch-ups in `01` / `03` / `05`.

Research notes were **not** fully rewritten; banners + path/readiness fixes only.

## Recommended next commits

1. **`docs: appliance operator guides`** — rebuilt `api`, `backend`, `configuration`, `contributing`, `development`, `installer`, `logging`, `release-process`, `research`, `roadmap` + `doc-build/*.research.md`.
2. **`docs: ADR 0001 and index`** — `adr/0001-appliance-architecture.md`, `adr/README.md`, `INDEX.md`, `doc-build/STATUS.md`.
3. **`docs: phase-0 research path banners`** — leftover cleanup on `research/01–05`, `prior-art.md`, this STATUS.
4. *(optional)* **`docs: ADR 0002 Inno Setup`** — promote installer decision from research/04.

Do **not** commit application code in these docs-only commits.

## Summary

| Category | Result |
| --- | --- |
| PLAN rebuild targets | **11/11 PASS** |
| Research briefs | **11/11 present** |
| Stale monorepo language in rebuilt guides | **none** |
| Blockers for merge of docs branch | **none** |
