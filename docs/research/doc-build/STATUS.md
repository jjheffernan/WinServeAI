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
| Phase-0 research `01–05` / `prior-art.md` | Still use old `packages/*` paths (historical notes; not PLAN rebuild targets) |

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

1. **Phase-0 research path drift** — `research/01–05` and parts of `prior-art.md` still name `packages/launcher`, `packages/process`, `packages/backend`, etc. Operator guides are clean; deep dives are historical. Optional follow-up: add a one-line banner (“paths predate single-crate layout; map to `app/src/…`”) or light path renames.
2. **Open research checklist items** — migration strategy, rotating logs, metrics endpoint, UI topics (intentional; not blocking doc-build).
3. **Readiness probe wording** — architecture uses `/v1/models`; research/02 prefers `/health`. Rebuilt `api.md` / `backend.md` document both; keep consistent in code when implementing.
4. **ADR backlog** — 0001 notes pin policy, Windows stop semantics, and installer choice as future ADRs (not required for this pass).

## Recommended next commits

1. **`docs: appliance operator guides`** — rebuilt `api`, `backend`, `configuration`, `contributing`, `development`, `installer`, `logging`, `release-process`, `research`, `roadmap` + `doc-build/*.research.md`.
2. **`docs: ADR 0001 and index`** — `adr/0001-appliance-architecture.md`, `adr/README.md`, `INDEX.md`, `doc-build/STATUS.md`.
3. *(optional)* **`docs: banner phase-0 research paths`** — note single-crate mapping on `research/01–05` without full rewrite.
4. *(optional)* **`docs: ADR 0002 Inno Setup`** — promote installer decision from research/04.

Do **not** commit application code in these docs-only commits.

## Summary

| Category | Result |
| --- | --- |
| PLAN rebuild targets | **11/11 PASS** |
| Research briefs | **11/11 present** |
| Stale monorepo language in rebuilt guides | **none** |
| Blockers for merge of docs branch | **none** |
