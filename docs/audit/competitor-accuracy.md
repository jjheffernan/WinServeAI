# Competitor research — independent accuracy review

**Date:** 2026-07-30  
**Scope:** Pinned competitor briefs, architecture synthesis, and WinServeAI completion audit  
**Branch:** `dev`  
**Reviewers:** independent agents (did not author the briefs they reviewed)

## Method

- Resolve each brief’s tag → commit SHA via GitHub API and confirm against the brief header.
- Existence-check cited paths at the pinned SHA; open line-numbered claims for spot checks.
- Cross-check `docs/research/06-competitor-architecture.md` and `STATUS.md` against the briefs only (no new upstream research).
- Spot-check `docs/audit/completion-audit.md` against `app/src/**`, `apps/desktop/`, `.github/workflows/ci.yml`, and backlog docs.

Confidence vocabulary remains that of [competitors/PLAN.md](../research/competitors/PLAN.md). One reviewer temporarily used `verified from release assets` for GitHub Release zip contents; that is treated here as **verified in source** for comparison purposes (asset listing + inspected zip membership at the pin).

## Per-product verdicts

| Artifact | Pin | Verdict | Material fixes applied |
| --- | --- | --- | --- |
| `ollama.research.md` | `v0.32.5` | **PASS** | None |
| `jan.research.md` | `v0.8.4` | **PASS_WITH_FIXES** | Corrected spawn-site parenthetical (`deps_analyzer` re-execs Jan, not backend); fixed preset/port/API-key citation overreach in Lifecycle Start |
| `llama-cpp-server.research.md` | `b9866` | **PASS_WITH_FIXES** | Packaging confidence clarified for release assets; CMake link evidence made precise (transitive `llama`) |
| `localai.research.md` | `v4.7.1` | **PASS_WITH_FIXES** | Docker images not present as GitHub Release assets — packaging claim narrowed to Dockerfiles/workflows |
| `gpt4all.research.md` | `v3.10.0` | **PASS** / minor fixes | No material accuracy failures; Unix-gated API tests already noted |
| `06-competitor-architecture.md` | n/a | **PASS_WITH_FIXES** | Demoted overclaims (desktop absence, GPT4All “readiness”, stop spectrum, test-strategy judgment); fixed Jan proxy diagram to in-process; scoped Ollama Job Object note |
| `competitors/STATUS.md` | n/a | **PASS_WITH_FIXES** | GPT4All evidence notes; recommended-next updated after this review; source-index gap recorded then serialized into `SOURCES.md` |
| `completion-audit.md` | `dev` @ `805f908` | **PASS** | None |

## Residual uncertainties (accepted for comparison)

These remain **inferred** / **unverified** and must not be treated as hard facts in `docs/comparison.md`:

1. Ollama: repository-wide Job Object / cgroup on the llama-server runner path; abnormal parent-death reclamation tests.
2. Jan: minimum `llama-server` build for `--models-preset`; abnormal Tauri-host death reclamation beyond PID sweep.
3. LocalAI: platform-specific process-manager Job Object/cgroup absence; machine-wide launcher singleton outside cited path.
4. GPT4All: no readiness probe in the pinned brief; embedded API pytest harness is Unix-gated.

## Source index

Missing CDN/model-host URLs flagged by the cross-report review were appended to [policies/SOURCES.md](../policies/SOURCES.md) (Jan CDN, Hugging Face, GPT4All gallery, `nomic-ai/llama.cpp`).

## Overall accuracy gate

**PASS for comparison.** No FAIL verdicts remain. Material citation errors were fixed in place. Comparison and roadmap work may proceed using only `verified in source` claims as hard facts; residual uncertainties above stay as explicit gaps.

## Raw reviewer notes

Orchestrator working copies (not published): `/tmp/accuracy-ollama-jan.md`, `/tmp/accuracy-llama-localai-gpt4all.md`, `/tmp/accuracy-cross-completion.md`.
