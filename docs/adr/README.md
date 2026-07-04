# Architecture Decision Records

ADRs capture **why** we chose a path, not how to implement it. Implementation detail lives in [architecture.md](../architecture.md) and the phase-0 research notes under [research/](../research/).

## Index

| ADR | Title | Status |
| --- | --- | --- |
| [0001](0001-appliance-architecture.md) | Appliance architecture | Accepted |

## Decisions locked elsewhere (no ADR yet)

| Decision | Summary | Where |
| --- | --- | --- |
| YAML config | Human-readable source of truth; no free-form CLI flags in config | [configuration.md](../configuration.md) |
| Inno Setup first | Preferred Windows installer starting point | [installer.md](../installer.md), [research/04-installer-licensing.md](../research/04-installer-licensing.md) |
| Localhost default | LAN bind and firewall are opt-in | [research/04-installer-licensing.md](../research/04-installer-licensing.md) |

Candidates for future ADRs: llama.cpp pin policy, Windows stop semantics (Job Object + CTRL_BREAK), installer SKU (CPU vs CUDA).

## Adding an ADR

1. Create `docs/adr/NNNN-short-title.md` (zero-padded number).
2. Use sections: **Status**, **Context**, **Decision**, **Consequences**.
3. Link it from this index.
4. Do not restate the full architecture; link to it.

Template:

```markdown
# NNNN. Title

- Status: Proposed | Accepted | Superseded
- Date: YYYY-MM-DD

## Context

## Decision

## Consequences
```
