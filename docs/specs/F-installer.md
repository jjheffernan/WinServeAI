# F — Installer (Phase 3 build spec)

**Milestone:** [PLAN.md](../PLAN.md) F1–F4 · **Phase:** [roadmap.md](../roadmap.md) Phase 3  
**Status:** Spec only — no Inno script implementation in this slice.  
**Research:** [research/04-installer-licensing.md](../research/04-installer-licensing.md), [installer.md](../installer.md)

## Goal

Ship an Inno Setup installer so a Windows user can **Install → Start → API available** without cloning the repo or installing Rust.

```text
Installer (Inno)
  → {app}\winserve.exe (+ tray if Phase 2 shipped)
  → {app}\bin\llama-server.exe (+ CUDA DLLs)
  → {app}\config\default.yaml
  → {app}\notices\…
Shortcut → manager/tray (never llama-server alone)
```

## Non-goals

* Model weights in the payload (GGUF licenses are separate; user supplies path)
* Auto-updater (Phase 5+)
* MSI/MSIX as MVP (Inno first; WiX/MSIX later if needed)
* Firewall rule for loopback-only default bind
* Chat UI or download helpers in the installer wizard beyond **model path** / basic settings (first-run may set `model.path` only)

## F1 — Inno layout and payload

### On-disk layout

Align with [installer.md](../installer.md) (operator guide). Prefer this over older `bin/winserve.exe` sketches in research notes.

```text
{app}\                              # e.g. C:\Program Files\WinServeAI
  winserve.exe                      # Server Manager CLI / headless entry
  winserve-tray.exe                 # Phase 2 desktop; Start Menu / desktop target when present
  bin\
    llama-server.exe                # pinned llama.cpp build (b#### from release docs)
    cudart64_XX.dll                 # CUDA build only; beside exe
    cublas*.dll                     # if linked; NVIDIA Attachment A only
  config\
    default.yaml                    # localhost bind; model.path placeholder or empty guidance
  logs\                             # optional empty dir, or created at runtime
  notices\
    THIRD_PARTY_NOTICES.md
    llama.cpp-LICENSE.txt
    llama.cpp-AUTHORS.txt
    NVIDIA-CUDA-NOTICE.txt          # CUDA builds only
  unins000.exe                      # Inno uninstaller
```

| Path | Role |
| --- | --- |
| `{app}\winserve.exe` | Manager binary (same crate as dev) |
| `{app}\winserve-tray.exe` | Desktop owner when Phase 2 exists; **preferred shortcut target** |
| `{app}\bin\` | External backend + private DLLs; no PATH mutation |
| `{app}\config\` | Human YAML source of truth |
| `{app}\notices\` | License compliance payload |

### Inno project location

```text
installer/
  README.md
  inno/
    WinServeAI.iss          # primary script (to implement)
    files/                  # optional staging notes
```

### `[Setup]` essentials

| Directive | Value / guidance |
| --- | --- |
| `AppName` | WinServeAI |
| `AppVersion` | release version (match cargo / tag) |
| `DefaultDirName` | `{autopf}\WinServeAI` |
| `DefaultGroupName` | WinServeAI |
| `PrivilegesRequired` | `admin` (Program Files + optional firewall) |
| `ArchitecturesAllowed` | `x64compatible` (match shipped binaries) |
| `OutputBaseFilename` | e.g. `WinServeAI-{version}-setup` |

Use `{auto*}` constants for shortcuts so admin vs per-user modes stay consistent ([Inno admin modes](https://jrsoftware.org/ishelp/topic_admininstallmode.htm)).

### `[Files]`

* Ship release-built `winserve.exe` (and tray when available).
* Ship pinned `bin/llama-server.exe` + DLL set from release packaging (`dumpbin /dependents` to list CUDA deps).
* Ship `config/default.yaml` (do not overwrite user-modified config on upgrade — use `onlyifdoesntexist` or equivalent).
* Ship entire `notices\*` tree.
* Never ship `.gguf` models.

### Pin

Installer embeds the same **b####** pin documented in [bin/README.md](../../bin/README.md) / [release-process.md](../release-process.md) at release time. Record pin in `notices/THIRD_PARTY_NOTICES.md`.

## F2 — Shortcuts

| Icon | Target | WorkingDir |
| --- | --- | --- |
| Start Menu `{group}\WinServeAI` | `{app}\winserve-tray.exe` if present, else `{app}\winserve.exe` | `{app}` |
| Desktop (optional `[Tasks]` `desktopicon`) | same | `{app}` |
| Uninstall | `{uninstallexe}` | n/a |
| Optional: Third-party notices | `{app}\notices\THIRD_PARTY_NOTICES.md` | n/a |

**Rules**

* Shortcuts launch the **manager/tray**, never `bin\llama-server.exe` as the primary entry.
* `WorkingDir: "{app}"` on every app icon so relative `config/` and `bin/` resolve.
* Optional desktop icon defaults **off** or on per product preference; document in ISS `[Tasks]`.

## F3 — Firewall rules

Default config binds **loopback** (`127.0.0.1`). **Do not** add a firewall rule for that case.

| Bind | Firewall rule? |
| --- | --- |
| `127.0.0.1` / `localhost` | **No** |
| `0.0.0.0` / LAN interface | **Yes** — inbound allow for `{app}\bin\llama-server.exe` |

Program-based rule (port-independent):

```text
netsh advfirewall firewall add rule name="WinServeAI llama-server" dir=in action=allow program="{app}\bin\llama-server.exe" enable=yes profile=any
```

Uninstall (must match name):

```text
netsh advfirewall firewall delete rule name="WinServeAI llama-server"
```

Wire via Inno `[Run]` / `[UninstallRun]`: `Filename: "{sys}\netsh.exe"`, `Flags: runhidden`. Gate add behind optional task **“Allow LAN access”** defaulting **off**, or only when first-run wizard selects non-loopback bind.

If the user changes bind after install, **Server Manager** (Phase 2+) should add/remove the same named rule so runtime stays in sync ([research/04](../research/04-installer-licensing.md)).

## F4 — Notices (`THIRD_PARTY_NOTICES`)

llama.cpp is MIT. Binary distributions must include copyright + permission notice in the **install payload** ([ollama#3185](https://github.com/ollama/ollama/issues/3185)).

**Must ship under `{app}\notices\`:**

1. `THIRD_PARTY_NOTICES.md` — rollup: component, upstream URL, license type, **pinned tag**
2. Full llama.cpp `LICENSE` text (`llama.cpp-LICENSE.txt`)
3. Prefer `AUTHORS` from the pinned tag
4. NVIDIA CUDA note when shipping Attachment A DLLs
5. Any other bundled OSS (MSVC runtime policy TBD — static link vs `vcredist`)

**Refresh process:** on every `bin/llama-server.exe` pin bump, copy LICENSE/AUTHORS and regenerate rollup. Keep a monorepo copy under `notices/` for release CI to copy into the installer stage.

**Must not:** rely only on GitHub README credit or source-tree LICENSE without install-tree copies.

## CUDA / DLL bundling

* Place DLLs **beside** `llama-server.exe` in `{app}\bin\`.
* Redistribute only NVIDIA **Attachment A** files ([CUDA EULA](https://docs.nvidia.com/cuda/eula/)).
* End users still need a compatible **GPU driver** (not redistributed).
* Optional later: split CPU vs CUDA installer SKUs if size hurts; MVP may ship one CUDA build + CPU fallback documented.

## Acceptance criteria

1. Clean install on Windows 10/11 x64 to Program Files (admin).
2. Start Menu (and optional desktop) shortcut starts **manager/tray**, not `llama-server` alone.
3. With `model.path` set to a user GGUF (wizard or Settings), user can reach **Ready** and `GET /v1/models` returns 200.
4. Default install uses loopback bind and installs **no** firewall rule.
5. Optional LAN task (or non-loopback config) creates program rule for `llama-server.exe`; uninstall deletes it.
6. `{app}\notices\THIRD_PARTY_NOTICES.md` and full llama.cpp license text are present after install.
7. Uninstall removes app files, shortcuts, and firewall rule (if any); does not delete user GGUF files outside `{app}`.
8. Upgrade path does not clobber a user-edited `config\default.yaml` without confirmation/policy.
9. No models, chat UI, or downloaders in the installer payload.

## Implementation order (when coding)

1. Stage release layout under `installer/inno/` (or CI artifact dir).
2. Author `WinServeAI.iss` with `[Files]`, `[Icons]`, `[Tasks]`.
3. Add notices generation from pin.
4. Optional LAN firewall `[Run]` / `[UninstallRun]`.
5. Smoke: install on clean VM → shortcut → set model path → Ready → uninstall.

## File touch list (expected implementation)

| Path | Change |
| --- | --- |
| `installer/inno/WinServeAI.iss` | Primary Inno script |
| `installer/README.md` | Build/sign instructions |
| `notices/` *(repo)* | `THIRD_PARTY_NOTICES.md` + license texts for pin |
| `bin/README.md` / `docs/release-process.md` | Pin recorded for release packaging |
| CI workflow *(optional)* | Build release binaries + compile ISS |
| `docs/installer.md` | Point at ISS once real |
| `docs/PLAN.md` | Check F1–F4 when done |

Coordinate shortcut target with [E-desktop.md](E-desktop.md) (tray vs CLI-only if Phase 2 slips).

## See also

- [PLAN.md](../PLAN.md) — F1–F4
- [installer.md](../installer.md) — operator packaging goals
- [research/04-installer-licensing.md](../research/04-installer-licensing.md) — Inno, firewall, CUDA, MIT
- [release-process.md](../release-process.md) — pin policy
- [E-desktop.md](E-desktop.md) — tray as shortcut target
- [A2-smoke.md](A2-smoke.md) — post-install API proof pattern
