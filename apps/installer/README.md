# Installer

Native Windows installer for one-click setup.

## Goals

- Desktop shortcut
- Start menu entry
- Firewall rules (LAN / localhost modes)
- Bundled `llama-server` backend
- First-run wizard (Phase 3)
- Auto updates (Phase 5, via `apps/updater`)

## Research (Phase 0)

| Tool | Notes |
| --- | --- |
| **Inno Setup** | Preferred starting point: simple, scriptable, widely used |
| NSIS | Mature, more verbose scripting |
| WiX | MSI-native, steeper learning curve |
| MSIX | Modern packaging; store-friendly, more constraints |

Decision is recorded in `docs/installer.md` once Phase 0 freezes architecture.

## Layout

```text
apps/installer/
├── README.md
└── inno/           # Inno Setup scripts (Phase 3)
```
