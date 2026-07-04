# Documentation drift enforcement

**Purpose:** Keep readiness scores, technical claims, and architecture rules from diverging from code and from each other.

## Sources of truth

| Claim type | Canonical source | Must match |
| --- | --- | --- |
| Architecture rules | `docs/architecture.md`, `docs/adr/0001-*.md` | `AGENTS.md`, operator guides |
| Module readiness scores | `docs/readiness/<id>.md` | `docs/readiness/README.md` dashboard **and** `> **Readiness:**` blocks in primary docs |
| Overall maturity | Average in `readiness/README.md` | `docs/TODO.md` header, `docs/PLAN.md` header |
| Config schema | `app/src/server/config.rs` + `config/default.yaml` | `docs/configuration.md` |
| Readiness probe | `app/src/server/health.rs` | `docs/api.md`, `docs/research/02-*.md` (primary = `/v1/models`) |
| llama flags | `app/src/runtime/llama.rs` only | Never in YAML docs as user-facing flags |
| Branch policy | `docs/release-process.md` | `README.md`, `contributing.md`, `development.md`, `AGENTS.md` |
| Implementation backlog | `docs/PLAN.md` / `docs/TODO.md` | Must not invent features absent from PLAN |

## Readiness score rules

1. **Single write path:** Scores are authored only in `docs/readiness/<id>.md` (dimension table + overall).
2. **Dashboard is derived:** `docs/readiness/README.md` overall/label/top-gap must equal the scorecard files.
3. **Operator banners are derived:** Primary docs listed in `readiness/PLAN.md` must contain a block:

   ```markdown
   > **Readiness:** … X.X/5 (`label`) — details in [readiness/<id>.md](…)
   ```

   Numbers and labels must match the scorecard(s) referenced.
4. **Refresh trigger:** Any material change under `app/src/`, `config/`, `installer/`, or `bin/README.md` requires a readiness refresh before merge to `dev` (or run `scripts/check_doc_drift.py` and fix failures).
5. **No silent edits:** Do not change a banner score without updating the scorecard and dashboard in the same commit.

## Technical claim rules

1. **Cite code or research:** Non-obvious behavior (probes, flags, Windows APIs) must link to a path (`app/src/…`) or research note / upstream URL. Prefer the URL index in [SOURCES.md](./SOURCES.md) for external links.
2. **Primary readiness probe** is `GET /v1/models` until `health.rs` changes. `/health` may be documented only as optional alternate.
3. **No `packages/*` as current layout** except historical path-note banners in research notes.
4. **No backend traits / plugins** as planned work unless ADR supersedes 0001.

## Enforcement

```bash
python3 scripts/check_doc_drift.py
```

Exit code `0` = pass, `1` = drift found (CI should fail).

CI: run on `dev` and `main` pushes/PRs (see `.github/workflows/ci.yml`).

## Agent obligations

When an agent edits docs or readiness:

1. Run `check_doc_drift.py` before finishing.
2. If scores change, update scorecard → dashboard → banners in one change set.
3. Prefer linking existing research over duplicating long explanations.
