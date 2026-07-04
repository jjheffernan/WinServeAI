# Agent Skills (agent-agnostic)

All project skills live in [`skills/`](skills/). This follows the [Agent Skills](https://agentskills.io) standard and works with Cursor, Claude Code, Codex, and other compatible agents.

## Installed

| Skill | Source |
| --- | --- |
| caveman, caveman-*, cavecrew | [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman) |
| ponytail, ponytail-* | [DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail) |
| grill-me, grilling, handoff | [mattpocock/skills](https://github.com/mattpocock/skills) |
| design-an-interface, ubiquitous-language | [mattpocock/skills](https://github.com/mattpocock/skills) |

## Policy

* **Single source of truth:** `.agents/skills/` only
* Do **not** install into per-agent directories (`.claude/skills`, `.windsurf/skills`, …)
* Lockfile: `skills-lock.json` at repo root
* Restore: `npm run skills:restore`
* Update: `npm run skills:update`

Install commands use `--agent cursor` so the CLI writes into `.agents/skills/` without fan-out to every agent vendor path. Skills themselves remain agent-agnostic.
