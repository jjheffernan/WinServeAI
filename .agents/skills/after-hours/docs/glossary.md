# After-hours loop — glossary & bootstrap

Modular overnight / late-session agent work. Orchestrator: `.agents/skills/after-hours-loop/SKILL.md` (see heff-skills SoT `skills/after-hours-loop/SKILL.md`).

## Ubiquitous language

| Term | Meaning |
|------|---------|
| **Loop run** | One armed session from bootstrap until stop |
| **FOR (sentinel)** | Cadence wakes only — sleep/echo or Automation cron |
| **WHILE (per wake)** | Guarded queue work; default one claim; interrupt = inner break |
| **Interrupt** | IDE/tool abort mid-item — park `blocked`/`interrupted`; keep sentinel |
| **Tick** | One orchestrator iteration: pick item → execute → record outcome |
| **Work item** | Normalized unit the loop consumes (portable contract: `id`, `title`, `acceptance`, `blockerPolicy`, `executorHint`, `outcomeKind` — see state-schema) |
| **Work source** | Adapter that materializes or refreshes work items from a tracker **input** |
| **Executor** | Strategy that completes one work item (or one slice) and emits a completion signal |
| **A→Z** | Executor-defined completion for an item (acceptance satisfied) plus a successful outcome adapter — **not** solely “opened a PR” |
| **Outcome adapter** | How completion is published (`draft-pr`, `doc-artifact`, `branch-only`, `report-only`, `external-ticket-update`) — separate from executors |
| **Outcome** | Item `status` after tick: `done`, `blocked`, or `skipped` |
| **Slice** | Smallest shippable unit for an executor (often one draft PR or one doc artifact) |
| **Agent-ready** | Work with clear acceptance (e.g. `ready-for-agent` + brief); AFK must not invent scope |
| **Sources** | Night-time binding to which trackers / inboxes feed this run — only binding; do not fork orchestration |

## Layers

| Layer | Modules (v1) |
|-------|----------------|
| Work source | `github-issues`, `todo-md`, `feature-spec`, `wayfinder-afk` (opt-in), `github-tickets` (opt-in) |
| Executor | `pr-slice`, `feature-build`, `research-only`, `docs-digest`, `ops-checklist` |
| Outcome adapter | `draft-pr`, `doc-artifact`, `branch-only`, `report-only`, `external-ticket-update` — [outcomes.md](../references/outcomes.md) |
| Orchestrator | `after-hours` skill + `/after-hours`, `/loop`, or Cursor Automation |
| References | readiness, compatibility, bootstrap, guardrails, state, outcomes, morning-brief, cloud-ledger |

## Resolved defaults

| Decision | Choice |
|----------|--------|
| Feature granularity | **Hybrid** — pre-split when plan lists slices; else umbrella + runtime child items |
| Queue priority | **GitHub-first** — drain issues before TODO / feature items |
| TODO write-back | **Auto check-off** — same PR branch flips `- [ ]` → `- [x]` |
| PRs | **Draft by default** overnight (`draftPrs: true`) |
| Base branch | **Configurable** (`baseBranch`, default `main`) |
| Automation | **Prompt per run** — edit Sources in Automations UI each night (template optional) |

## Canonical work item

```json
{
  "id": "github:52",
  "title": "Example issue title",
  "acceptance": "optional but preferred",
  "blockerPolicy": "block",
  "executorHint": "pr-slice",
  "outcomeKind": "draft-pr",
  "source": "github-issues",
  "executor": "pr-slice",
  "ref": "https://github.com/owner/name/issues/52",
  "granularity": "single-pr",
  "status": "open"
}
```

**ID convention:** `{source}:{stable-key}` — e.g. `github:52`, `todo:3b-pause-audit`, `feature:example-phase1`.  
Portable contract + aliases: [state-schema.md](../references/state-schema.md).

## Bootstrap (in-session)

```text
/after-hours 45m
Sources:
  - github-issues: label ready-for-agent limit 5
  - todo-md: section "Now"
  - feature-spec: docs/plans/example-plan.md phase 1
maxPrs: 3
priority: github-first
```

Equivalent: `/loop 45m` followed by “Follow `.agents/skills/after-hours-loop/SKILL.md`” plus the same Sources block.

Run bootstrap immediately (tick 0), then arm sentinel `AGENT_LOOP_TICK_AFTERHOURS`.

## Bootstrap (Cursor Automation)

```text
/after-hours bootstrap:
  - github-issues: label ready-for-agent limit 5
  - todo-md: section "Now"
maxPrs: 2
priority: github-first
```

Checkout configured **`baseBranch`**.

## State & morning brief

| Artifact | Path (default) | Gitignore |
|----------|----------------|-----------|
| State | `.cursor/after-hours-loop.state.json` | Yes |
| Morning brief | `.cursor/after-hours-morning-brief.md` | Yes |
| Cloud ledger | config `cloudLedgerPath` (default off / `null`; e.g. `.cursor/after-hours-ledger.json`) | No — must be committed when enabled |
| Config | `.cursor/after-hours-loop.config.json` | Optional (often commit) |

## Stop

- Empty queue mid-run → **soft-park** (`parkedReason: empty-queue`); keep sentinel
- `maxPrs` reached mid-run → **soft-park** (`parkedReason: maxPrs`); keep sentinel
- Guardrail / preflight / blocked streak / CI stop / unrecovered dirty-interrupt → `blocked` + matching `stopDetail` (**Stop loop**)
- User: **stop after-hours** / **stop loop** → `done` / `user-stop` (kill **all** matching sentinel PIDs)
- IDE **interrupt** mid-item → **not** a stop — park item `interrupted`; sentinel stays armed ([tick-and-runners](../references/tick-and-runners.md))
- `/after-hours Nm` → tick **interval** only, never total wall-clock budget

On true stop, write the morning brief (pointers to PRs, non-PR outcomes, and blocked items). Coarse stop enum: [state-schema.md](../references/state-schema.md).
