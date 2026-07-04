# Installer & Licensing

> **Path note (appliance layout):** This note was written against an earlier `packages/*` monorepo. Map old paths to the current single crate:
> `packages/launcher` → `app/src/server/manager.rs` · `packages/process` → `app/src/runtime/process.rs` · `packages/llama` → `app/src/runtime/llama.rs` · `packages/api` / readiness → `app/src/server/health.rs` + `app/src/api/` · `packages/hardware` → `app/src/system/` · `packages/config` → `app/src/server/config.rs` · `packages/logging` → `app/src/server/logs.rs` · `packages/backend` / traits → **removed** (no backend trait).

Phase 0 research for Windows packaging: Inno Setup, firewall, CUDA DLL bundling, MIT notices. Builds on [installer.md](../installer.md), [prior-art.md](../prior-art.md), installer scripts under `apps/installer` (if present). Pinned `llama-server` lives in **`bin/`** (version pin in release docs), not a `vendor/llama.cpp` tree. No production code.

---

## Recommendations for installer and `bin/`

### Inno Setup (`apps/installer/inno/`)

Preferred tool ([docs/installer.md](../installer.md)). Phase 3 scripts live under `apps/installer/inno/` (stub today).

| Concern | Pattern | Notes |
| --- | --- | --- |
| Start Menu | `[Icons]` → `Name: "{group}\WinServeAI"` | `{group}` = Start Menu folder from `DefaultGroupName`. Docs: [Icons section](https://jrsoftware.org/ishelp/topic_iconssection.htm) |
| Desktop shortcut | `Name: "{autodesktop}\WinServeAI"; Tasks: desktopicon` | Optional via `[Tasks]`. Prefer `{auto*}` constants so admin vs per-user install modes stay consistent ([Inno 6 admin modes](https://jrsoftware.org/ishelp/topic_admininstallmode.htm), [SO: admin vs user shortcuts](https://stackoverflow.com/questions/50832956/inno-setup-5-6-0-warning-about-shortcuts-and-admin-v-user)) |
| Working dir | `WorkingDir: "{app}"` on every icon | Avoids broken relative paths when launching manager/tray |
| Uninstall entry | `Name: "{group}\Uninstall WinServeAI"; Filename: "{uninstallexe}"` | Standard Inno pattern |
| Elevation | `PrivilegesRequired=admin` (or `PrivilegesRequiredOverridesAllowed`) | Needed for Program Files + `netsh advfirewall`. Do **not** mix `{userdesktop}` with admin install |

**Firewall (`netsh advfirewall`)** — only when bind is non-loopback (see below):

```text
# Install (program-based inbound allow — preferred)
netsh advfirewall firewall add rule name="WinServeAI llama-server" dir=in action=allow program="{app}\bin\llama-server.exe" enable=yes profile=any

# Uninstall (must match name)
netsh advfirewall firewall delete rule name="WinServeAI llama-server"
```

Wire via `[Run]` / `[UninstallRun]` with `Flags: runhidden`, `Filename: "{sys}\netsh.exe"`, and check exit codes ([SO: Inno + netsh](https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception), [SO: advfirewall syntax](https://stackoverflow.com/questions/31970310/running-netsh-from-using-inno-setup-exec)). Prefer **program** rules over port rules so port changes in YAML do not orphan rules. Optional: gate with a `[Tasks]` checkbox (“Allow LAN access”) defaulting off.

Reference community pattern: [d4-ollama-win-service](https://github.com/internetics-net/d4-ollama-win-service) (NSSM + firewall — Phase 5 service mode, not tray MVP).

### When to add firewall rules

| Bind | Firewall rule? | Rationale |
| --- | --- | --- |
| `127.0.0.1` / `localhost` (MVP default) | **No** | Loopback is not filtered by Windows Firewall inbound rules; opening a rule is noise and a false sense of “LAN ready” |
| `0.0.0.0` / LAN interface | **Yes**, inbound allow for `llama-server.exe` | Clients on other hosts need an inbound exception; reversible on uninstall |
| Bind changes post-install | Add/remove rule from **Server Manager**, not only installer | Installer can seed a rule only if first-run wizard chooses LAN; runtime must stay in sync |

Aligns with [docs/installer.md](../installer.md) and prior-art: **localhost default, firewall only for LAN** ([GPT4All localhost-only](https://github.com/nomic-ai/gpt4all/wiki/Local-API-Server), exposed-Ollama caution).

### CUDA DLL bundling (release `bin/` → install `bin/`)

Ship `llama-server.exe` **and** its CUDA runtime DLLs **in the same directory** (`{app}\bin\`). Windows loads DLLs from the exe directory first; relying on system `PATH` / full CUDA Toolkit is a support trap ([Visokio Windows CUDA notes](https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda-)).

**Build matrix (release packaging, not day-to-day commits):**

```text
bin/                     # repo / release artifact layout
├── llama-server.exe     # pinned upstream b#### build
├── cudart64_XX.dll      # CUDA build only; beside exe
└── …                    # other Attachment A DLLs as needed

# Optional release variants (packaging only):
#   cpu/llama-server.exe
#   cuda/llama-server.exe + CUDA DLLs
# Version pin + notices: docs/release-process.md / docs/backend.md + notices/
```

Installer copies the chosen variant into `{app}\bin\`. Pin toolkit version used to build; ship **exact** matching `cudart64_*.dll` (and any other Attachment A libs the binary links — often `cublas*`, `cublasLt*`, sometimes `cudart` only depending on build). Use `dumpbin /dependents` (or equivalent) on the release artifact to list required DLLs.

**NVIDIA redistributable rules (high level — not legal advice):**

- Redistribute only files listed in **Attachment A** of the [CUDA Toolkit EULA](https://docs.nvidia.com/cuda/eula/) (versioned names like `cudart64_12.dll` count).
- App must have **material additional functionality** beyond the SDK bits; DLLs may only be accessed by your app.
- Do **not** ship the full CUDA Toolkit/SDK or developer tools to end users.
- End users still need a compatible **NVIDIA GPU driver**; drivers are not redistributable via the CUDA EULA grant.
- Prefer bundling DLLs beside the exe over requiring users to install the toolkit ([NVIDIA forum: deployment](https://forums.developer.nvidia.com/t/cuda-application-deployment-what-is-correct-deployment/15459)).

Include a short NVIDIA notice in `notices/` (EULA pointer / “contains NVIDIA CUDA redistributables”) when shipping CUDA builds.

### MIT / third-party notices (llama.cpp)

llama.cpp is [MIT](https://github.com/ggml-org/llama.cpp/blob/master/LICENSE). MIT requires the copyright + permission notice in **all copies or substantial portions**, including **binary** distributions — source-repo `LICENSE` alone is not enough.

**Cautionary tale:** [ollama/ollama#3185](https://github.com/ollama/ollama/issues/3185) — binary install trees lacked notices; community backlash (HN: [item 44003741](https://news.ycombinator.com/item?id=44003741)). WinServeAI must ship notices **inside the installed payload**, not only in the monorepo.

Minimum for each release:

1. Full llama.cpp `LICENSE` text (copyright line as upstream ships — currently “ggml authors”).
2. Prefer also `AUTHORS` (or equivalent attribution list) from the pinned tag.
3. Notices for other bundled OSS (e.g. cpp-httplib if linked into `llama-server`, MSVC runtime if redistributed, NVIDIA note for CUDA DLLs).
4. Refresh notices whenever the `bin/llama-server.exe` pin changes (record tag in release docs).

Model weights are **out of scope** and have separate licenses ([llama.cpp discussion #472](https://github.com/ggml-org/llama.cpp/discussions/472)) — do not bundle models in the installer.

---

## Proposed install layout

```text
{app}/                          # e.g. C:\Program Files\WinServeAI
├── bin/
│   ├── winserve.exe            # Server Manager (process owner)
│   ├── winserve-tray.exe       # Phase 2; Start Menu / desktop target
│   ├── llama-server.exe        # pinned build from bin/
│   ├── cudart64_XX.dll         # CUDA build only; beside exe
│   └── cublas*.dll             # if linked; Attachment A only
├── config/
│   └── winserve.yaml           # default human YAML (localhost bind)
├── notices/
│   ├── THIRD_PARTY_NOTICES.md  # rollup index
│   ├── llama.cpp-LICENSE.txt
│   ├── llama.cpp-AUTHORS.txt
│   └── NVIDIA-CUDA-NOTICE.txt  # CUDA builds
└── unins000.exe                # Inno uninstaller
```

| Path | Role |
| --- | --- |
| `bin/` | All executables + private DLLs; no PATH mutation required |
| `config/` | Default YAML; user edits stay human-readable (`app/src/server/config.rs`) |
| `notices/` | License compliance payload; also link from About / Start Menu optional “Third-party notices” icon |

Shortcuts point at tray/manager in `bin/`, with `WorkingDir: "{app}"` or `"{app}\bin"` consistently. Do not put `llama-server.exe` on the Start Menu as a primary entry — users start the **manager**, which owns the backend.

---

## THIRD_PARTY_NOTICES requirements

Ship as `{app}\notices\THIRD_PARTY_NOTICES.md` (and keep a monorepo copy under `notices/` or repo-root `THIRD_PARTY_NOTICES.md` generated from the pin).

**Must include:**

- Component name, upstream URL, license type (MIT), pinned version/tag
- Full license text (not a link-only stub)
- Copyright holders as upstream states them

**Must travel with:**

- Installer payload (Inno `[Files]` → `notices\*`)
- Any portable/zip release of the same binaries
- Optional: `winserve --licenses` or tray “About” that opens `notices\` (nice-to-have; files on disk are the compliance floor)

**Must not:**

- Rely solely on GitHub README credit
- Strip notices from release CI artifacts
- Treat a source-tree LICENSE alone as sufficient for end-user installs

**Refresh process (minimal):** when bumping the `bin/llama-server.exe` pin (tag in release docs), copy `LICENSE` (+ `AUTHORS`) into `notices/` and regenerate the rollup. Automate later if painful ([prior-art open Q #9](../prior-art.md)).

---

## Links

| Topic | URL |
| --- | --- |
| Inno `[Icons]` | https://jrsoftware.org/ishelp/topic_iconssection.htm |
| Inno admin install mode | https://jrsoftware.org/ishelp/topic_admininstallmode.htm |
| Inno firewall via netsh | https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception |
| netsh `advfirewall firewall` syntax | https://stackoverflow.com/questions/31970310/running-netsh-from-using-inno-setup-exec |
| Netsh AdvFirewall reference | https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-r2-and-2008/dd734783(v=ws.10) |
| CUDA EULA (Attachment A) | https://docs.nvidia.com/cuda/eula/ |
| CUDA deploy / redistribute DLLs | https://forums.developer.nvidia.com/t/cuda-application-deployment-what-is-correct-deployment/15459 |
| Windows CUDA path pitfalls | https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda- |
| llama.cpp LICENSE | https://github.com/ggml-org/llama.cpp/blob/master/LICENSE |
| llama.cpp AUTHORS | https://github.com/ggml-org/llama.cpp/blob/master/AUTHORS |
| Ollama missing binary notices | https://github.com/ollama/ollama/issues/3185 |
| HN on notices | https://news.ycombinator.com/item?id=44003741 |
| Code MIT vs model licenses | https://github.com/ggml-org/llama.cpp/discussions/472 |
| NSSM + firewall prior art | https://github.com/internetics-net/d4-ollama-win-service |

---

## Open questions

1. **LAN rule timing** — Installer-only (wizard checkbox) vs Server Manager adds/removes rule whenever `host` changes in YAML?
2. **Program vs port rule** — Stick to program-based for `llama-server.exe`, or also allow a fixed-port rule for locked-down enterprise images?
3. **CPU vs CUDA installers** — One fat installer with both builds, two SKUs, or download CUDA DLLs on first GPU detect? (Fat install is simplest; size may push split SKUs.)
4. **Exact CUDA DLL set** — Confirm with `dumpbin` on the pinned `llama-server` CUDA build; document pin notes in release docs.
5. **MSVC runtime** — Static link vs ship `vcruntime`/`msvcp` (VC++ Redistributable merge module / `vcredist` bootstrap)?
6. **NVIDIA notice text** — Minimal EULA pointer in `notices/` sufficient, or any extra attribution NVIDIA expects for Attachment A files?
7. **Per-user install** — Can firewall rules work without admin (`PrivilegesRequired=lowest`)? If not, LAN mode requires elevation path.
8. **License automation** — Script to regenerate `THIRD_PARTY_NOTICES` from vendored trees on pin bump?
