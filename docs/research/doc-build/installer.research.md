# Installer research (doc-build)

Merged from [research/04-installer-licensing.md](../04-installer-licensing.md), [prior-art.md](../../prior-art.md) (installer / licensing / firewall), and [installer/README.md](../../../installer/README.md). Aligns with appliance architecture: single `winserve` binary owns `llama-server.exe`; no multi-backend packaging.

**Build target:** [docs/installer.md](../../installer.md)

---

## Sources

| Source | What it contributes |
| --- | --- |
| `installer/README.md` | Canonical install goals + on-disk layout (`winserve.exe`, `bin/llama-server.exe`, `config/default.yaml`, `THIRD_PARTY_NOTICES`) |
| `research/04-installer-licensing.md` | Inno patterns, firewall gating, CUDA DLL bundling, MIT / notices requirements, open questions |
| `prior-art.md` | Steal/avoid table; Inno + `netsh`; localhost default; ship notices with binaries (Ollama #3185); defer WiX/MSIX/NSSM |

---

## Tool choice

| Option | Pros | Cons | Decision |
| --- | --- | --- | --- |
| **Inno Setup** | Simple, scriptable, common for desktop apps | Not MSI-native | **Preferred** (Phase 3 starting point) |
| NSIS | Mature, flexible | Verbose scripting | Candidate only |
| WiX | Proper MSI | Steep learning curve | Enterprise later |
| MSIX | Store-friendly | Constraints, different update model | Future evaluation |

Scripts live under `installer/` (Inno preferred). Auto-updates are out of MVP (Phase 5+).

Prior-art P1: prefer Inno; firewall only for non-loopback; bundle manager + `llama-server.exe` + CUDA runtime DLLs **beside** the backend exe.

---

## Goals (MVP)

From `installer/README.md` + Phase 3 deliverable:

1. Install `winserve.exe` (Server Manager) and `bin/llama-server.exe` (+ private DLLs).
2. Desktop shortcut and Start Menu entry → launch **`winserve`**, not `llama-server.exe`.
3. Default config: `config/default.yaml` (localhost bind).
4. Firewall rule **only** when bind is non-loopback; reversible on uninstall.
5. Ship `THIRD_PARTY_NOTICES` (and full license texts) in the install payload.
6. Success: install → start via shortcut → OpenAI-compatible API on configured host/port.

Explicitly **not** MVP: auto-update, WiX/MSIX, NSSM/Windows Service, model bundles, chat UI.

---

## Layout on disk (installed)

Canonical layout from `installer/README.md`, reconciled with architecture (`bin/`, `config/`, `logs/`) and licensing research (`notices/` for full texts):

```text
%ProgramFiles%\WinServeAI\          # {app} — e.g. C:\Program Files\WinServeAI
  winserve.exe                      # Server Manager (process owner); shortcut target
  bin\
    llama-server.exe                # from vendor/llama.cpp pin
    cudart64_XX.dll                 # CUDA build only; beside exe
    cublas*.dll                     # if linked; NVIDIA Attachment A only
  config\
    default.yaml                    # human YAML; localhost bind by default
  logs\                             # created at runtime (server / llama / error)
  notices\
    THIRD_PARTY_NOTICES.md          # rollup index (or THIRD_PARTY_NOTICES.txt at {app})
    llama.cpp-LICENSE.txt
    llama.cpp-AUTHORS.txt
    NVIDIA-CUDA-NOTICE.txt          # CUDA builds
  unins000.exe                      # Inno uninstaller
```

| Path | Role |
| --- | --- |
| `{app}\winserve.exe` | Only primary entry point; Start Menu / desktop icons target this |
| `{app}\bin\` | Backend + private DLLs; no PATH mutation |
| `{app}\config\` | Default YAML; user-editable |
| `{app}\logs\` | Unified logs (runtime) |
| `{app}\notices\` | License compliance payload |

**Naming note:** older research used `winserve-launcher.exe` under `bin/` and `apps/installer`. Appliance layout uses root `winserve.exe` and repo `installer/`. Shortcuts must not expose `llama-server.exe` as the primary entry — users start the manager, which owns the backend.

**Inno icon pattern:**

- Start Menu: `[Icons]` → `Name: "{group}\WinServeAI"`; `Filename: "{app}\winserve.exe"`; `WorkingDir: "{app}"`
- Desktop (optional task): `Name: "{autodesktop}\WinServeAI"`; same filename / working dir
- Uninstall: `Name: "{group}\Uninstall WinServeAI"`; `Filename: "{uninstallexe}"`
- Prefer `{auto*}` constants with admin install mode; set `WorkingDir: "{app}"` on every icon

Elevation: `PrivilegesRequired=admin` (or overrides) for Program Files + `netsh advfirewall`. Do not mix `{userdesktop}` with admin install.

---

## Firewall (non-loopback only)

| Bind | Firewall rule? | Rationale |
| --- | --- | --- |
| `127.0.0.1` / `localhost` (MVP default) | **No** | Loopback is not filtered by Windows Firewall inbound rules; a rule is noise and a false “LAN ready” signal |
| `0.0.0.0` / LAN interface | **Yes** | Inbound allow for `llama-server.exe`; delete on uninstall |
| Bind changes post-install | Server Manager should add/remove rule when `host` changes | Installer may seed a rule only if wizard chooses LAN |

Prefer **program-based** rules over port rules so YAML port changes do not orphan rules:

```text
# Install (only when non-loopback)
netsh advfirewall firewall add rule name="WinServeAI llama-server" dir=in action=allow program="{app}\bin\llama-server.exe" enable=yes profile=any

# Uninstall (must match name)
netsh advfirewall firewall delete rule name="WinServeAI llama-server"
```

Wire via Inno `[Run]` / `[UninstallRun]`, `Flags: runhidden`, `Filename: "{sys}\netsh.exe"`, check exit codes. Optional `[Tasks]` checkbox (“Allow LAN access”) defaulting **off**.

Prior-art: GPT4All localhost-only default; exposed-Ollama caution; d4-ollama-win-service is NSSM + firewall for **service** mode (Phase 5), not tray MVP.

---

## CUDA / backend bundling

Ship `llama-server.exe` and its CUDA runtime DLLs in **`{app}\bin\`** (same directory). Windows loads DLLs from the exe directory first; relying on system `PATH` / full CUDA Toolkit is a support trap.

Release packaging (not day-to-day commits): pin `vendor/llama.cpp` tag; optional `cpu/` vs `cuda/` variants; installer copies chosen variant into `{app}\bin\`. Use `dumpbin /dependents` on the release artifact for the exact DLL set.

NVIDIA (high level, not legal advice): redistribute only Attachment A files from the CUDA Toolkit EULA; do not ship the full toolkit; end users still need a compatible GPU **driver**. Include a short NVIDIA notice when shipping CUDA builds.

Models are **out of scope** — separate licenses; do not bundle GGUF weights.

---

## THIRD_PARTY_NOTICES

MIT (llama.cpp) requires copyright + permission notice in **all copies or substantial portions**, including **binary** distributions. Source-repo `LICENSE` alone is not enough.

Cautionary: [ollama/ollama#3185](https://github.com/ollama/ollama/issues/3185) — binary install trees lacked notices.

**Must ship in install payload:**

1. Rollup `THIRD_PARTY_NOTICES` (component, URL, license type, pinned version/tag).
2. Full llama.cpp `LICENSE` text (copyright as upstream ships).
3. Prefer `AUTHORS` (or equivalent) from the pinned tag.
4. Notices for other bundled OSS (e.g. linked deps, MSVC runtime if redistributed, NVIDIA note for CUDA DLLs).

**Must travel with:** installer payload (`[Files]` → `notices\*`), any portable/zip of the same binaries.

**Must not:** rely only on GitHub README credit; strip notices from release artifacts; treat monorepo `vendor/` LICENSE as sufficient for end-user installs.

**Refresh:** when bumping `vendor/llama.cpp` pin, copy `LICENSE` (+ `AUTHORS`) and regenerate the rollup.

---

## Phase 3 deliverable

Install → user launches **winserve** (shortcut) → Server Manager starts `bin/llama-server.exe` → readiness (`/v1/models` or `/health`) → OpenAI-compatible API on configured host/port. Stop winserve to free GPU.

---

## Open questions (carry from research/04)

1. LAN rule timing — installer wizard only vs Server Manager syncs on config change?
2. Program vs port rule for locked-down enterprise images?
3. One fat installer (CPU+CUDA) vs split SKUs?
4. Exact CUDA DLL set for pinned build (`dumpbin`)?
5. MSVC runtime — static link vs `vcredist` bootstrap?
6. Per-user install without admin — can firewall rules work?
7. Automate `THIRD_PARTY_NOTICES` regeneration on pin bump?

---

## Links (key)

| Topic | URL |
| --- | --- |
| Inno `[Icons]` | https://jrsoftware.org/ishelp/topic_iconssection.htm |
| Inno admin install mode | https://jrsoftware.org/ishelp/topic_admininstallmode.htm |
| Inno + netsh firewall | https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception |
| CUDA EULA (Attachment A) | https://docs.nvidia.com/cuda/eula/ |
| llama.cpp LICENSE | https://github.com/ggml-org/llama.cpp/blob/master/LICENSE |
| Ollama missing binary notices | https://github.com/ollama/ollama/issues/3185 |

Full link tables remain in [research/04-installer-licensing.md](../04-installer-licensing.md) and [prior-art.md](../../prior-art.md).
