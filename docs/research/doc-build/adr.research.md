# ADR process research

Lightweight Architecture Decision Records for WinServeAI. No production code — process only.

**Canonical product docs:** [architecture.md](../../architecture.md), [vision.md](../../vision.md).  
**Deep dives (not ADRs):** [research/01–05](../), [prior-art.md](../../prior-art.md).

---

## Why ADRs here

WinServeAI is a small appliance wrapper. Most choices live in `architecture.md` and research notes. ADRs exist for **decisions that are hard to reverse** and that future contributors (human or agent) must not silently undo.

Research notes explore options. ADRs **lock a choice** and record why alternatives lost.

---

## When to write an ADR

Write one when **all** of the following hold:

1. The choice affects layout, process ownership, packaging, or the external boundary (`llama-server.exe`).
2. Reversing it would touch more than one module or force a release/vendor change.
3. A reasonable contributor might pick the other option without this record.

**Write ADRs for**

| Topic | Why |
| --- | --- |
| Appliance shape (one orchestrator, llama only) | Core identity; prevents platform creep |
| Pinning `llama-server` release | CLI/API drift; argv mapping is pin-specific |
| Windows stop / Job Objects | Orphan processes; OS-level guarantees |
| Installer technology (Inno vs WiX/MSI) | Packaging and enterprise distribution |

**Do not write ADRs for**

* Routine implementation details (log field names, timeout constants)
* MVP exclusions already listed in `vision.md` (chat UI, downloads, Docker, …)
* Open research still in `docs/research/0N-*.md` — promote to ADR only when accepted
* Style / tooling preferences unless they bind the release pipeline

If `architecture.md` already states the rule and nothing is contested, **update architecture** instead of adding an ADR. Use an ADR when the *trade-off* matters more than the rule itself.

---

## Lightweight format

One file per decision: `docs/adr/NNNN-short-kebab-title.md`.

```markdown
# NNNN. Title

- **Status:** Proposed | Accepted | Superseded by NNNN | Deprecated
- **Date:** YYYY-MM-DD

## Context

What forces the decision? Link research / prior-art.

## Decision

What we will do (one paragraph, imperative).

## Consequences

What becomes easier, harder, or forbidden. Point at code/docs owners.
```

Rules:

* **Short.** Prefer under ~80 lines. Link `docs/research/*` instead of pasting tables.
* **One decision per file.** Split if two independent choices.
* **Status is mandatory.** `Accepted` means implementers treat it as binding until superseded.
* **No backend-trait / plugin ADRs** while the appliance lock holds (`architecture.md`).

Numbering: zero-padded four digits, sequential. Do not renumber; supersede with a new ADR.

---

## Index (`docs/adr/README.md`)

The index is the only entry point agents should open first:

1. One-line process summary + link to this research note (optional).
2. **When to write** (condensed checklist).
3. **Accepted** table: number, title, status, one-line summary.
4. **Planned / placeholder** table for known upcoming decisions (no file yet).

Keep the index updated when adding or superseding an ADR.

---

## Planned ADRs (placeholders)

| Working title | Source research | Decision to lock |
| --- | --- | --- |
| Pin llama.cpp release | [02-llama-readiness.md](../02-llama-readiness.md) (`b####` tags, CLI drift) | How we pin, record, and bump `llama-server.exe` |
| Windows graceful stop | [01-windows-process.md](../01-windows-process.md) (Job Objects, `process-wrap`) | Cooperative stop + kill-on-manager-exit |
| Inno vs WiX | [04-installer-licensing.md](../04-installer-licensing.md), [installer.md](../../installer.md) | MVP installer technology |

These stay **placeholders in the index** until someone accepts a decision and adds `NNNN-….md`.

---

## First accepted ADR (optional build)

`0001-appliance-architecture.md` — summarize [architecture.md](../../architecture.md) as an Accepted ADR: one orchestrator (`ServerManager`), llama.cpp only, flags only in `runtime/llama`, YAML config, no multi-backend. Architecture remains the narrative; ADR 0001 is the decision record agents cite when rejecting platform-shaped PRs.

---

## Anti-patterns

* ADR per PR or per bugfix
* Duplicating full research tables into the ADR body
* “Proposed” ADRs that sit forever — either accept, reject (delete or mark Deprecated), or leave as research only
* ADRs that contradict `architecture.md` without superseding 0001 and updating architecture

---

## Build checklist

- [x] Process: when / format / index / placeholders
- [x] `docs/adr/README.md` — index + when-to-write + planned list
- [x] `docs/adr/0001-appliance-architecture.md` — Accepted, short
- [x] No `.gitkeep` present (directory has real files)
