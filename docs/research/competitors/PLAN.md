# Competitor architecture research — plan

**Date:** 2026-07-30  
**Branch:** `dev`  
**Scope:** Code-verifiable open-source competitors only (no proprietary-only products).

## Products in scope

| Product | Upstream | Brief |
| --- | --- | --- |
| llama.cpp server | https://github.com/ggml-org/llama.cpp | `llama-cpp-server.research.md` |
| Ollama | https://github.com/ollama/ollama | `ollama.research.md` |
| Jan | https://github.com/janhq/jan | `jan.research.md` |
| LocalAI | https://github.com/mudler/LocalAI | `localai.research.md` |
| GPT4All | https://github.com/nomic-ai/gpt4all | `gpt4all.research.md` |

**Out of scope:** LM Studio and other closed-source products (not code-verifiable).

## Confidence vocabulary (use exactly these words)

| Label | Meaning |
| --- | --- |
| `verified in source` | Claim backed by a specific path in a pinned upstream revision |
| `inferred from source` | Reasonable conclusion from nearby code/docs; not a direct quote of behavior |
| `unverified` | Marketing, README claims, or hearsay without a source path |

Comparisons and architecture synthesis may treat only `verified in source` as hard facts. Infer/unverified become explicit gaps.

## Rules for all research agents

1. Pin the inspected upstream tag or commit SHA in the brief header (`**Pinned:**`, `**Verified:** YYYY-MM-DD`).
2. Cite source paths for every architectural claim (process ownership, runtime boundary, model/config storage, API surface, lifecycle, desktop/service split, packaging, tests).
3. Prefer reading public GitHub trees / docs via API or fetch — do not invent internals.
4. Do **not** recommend WinServeAI grow multi-backend traits, chat UI, or model marketplaces.
5. Link to existing WinServeAI research (`docs/research/01`–`05`, `docs/prior-art.md`) instead of restating Job Object / readiness theory.
6. Every external URL used must later appear in `docs/policies/SOURCES.md` (orchestrator serializes after all briefs).
7. Brief length: enough for architecture comparison, not a product review.

## Brief template

```markdown
# <Product> — architecture research

**Upstream:** <url>
**Pinned:** <tag or commit>
**Verified:** YYYY-MM-DD
**License (source tree):** <SPDX or LICENSE path>

## Summary

<3–5 sentences: what the product is architecturally>

## Claims

| Claim | Evidence (path @ rev) | Confidence |
| --- | --- | --- |
| … | `path/to/file` @ `<sha-or-tag>` | verified in source |

## Process ownership

Who owns the inference process? Parent/child? Service? Job Objects / cgroup?

## Runtime boundary

What is “our code” vs external binary / library? Multiple backends?

## Model and config storage

Where models live; config format; downloads?

## API surface

OpenAI-compatible? Ports? Auth?

## Lifecycle

Start / stop / crash / single-instance behavior.

## Desktop vs service split

GUI, CLI, daemon, tray — how they relate.

## Packaging

Installer / brew / docker / portable.

## Test strategy

What automated tests exist (unit/integration/e2e) that prove ownership or API?

## Layout (text diagram)

```text
…
```

## Relevance to WinServeAI (facts only)

Steal / avoid notes must cite verified claims; no speculative parity wishlist.
```

## Orchestrator deliverables

1. Five `*.research.md` briefs (parallel agents).
2. `STATUS.md` roll-up after briefs + accuracy review.
3. `docs/research/06-competitor-architecture.md` synthesis + diagrams.
4. `docs/audit/competitor-accuracy.md` independent accuracy verdict.
5. `docs/comparison.md` WinServeAI vs families (after accuracy PASS).
