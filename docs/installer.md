# Installer

> **Readiness:** installer 3.0/5 (`mvp-partial`), bin 3.2/5 (`mvp-partial`) — details in [readiness/installer.md](./readiness/installer.md), [readiness/bin.md](./readiness/bin.md)

Native Windows packaging for the WinServeAI appliance. Install → launch
**`winserve-tray`** (fallback `winserve`) → set a local `.gguf` → OpenAI-compatible
API on the configured host/port.

Deep research: [research/04-installer-licensing.md](./research/04-installer-licensing.md), [research/doc-build/installer.research.md](./research/doc-build/installer.research.md). Build spec: [specs/F-installer.md](./specs/F-installer.md). Stub goals: [installer/README.md](../installer/README.md). Prior art: [prior-art.md](./prior-art.md). First run: [first-run.md](./first-run.md).

## Tool choice

| Option | Pros | Cons | Status |
| --- | --- | --- | --- |
| **Inno Setup** | Simple, scriptable, common for desktop apps | Not MSI-native | **Preferred starting point** |
| NSIS | Mature, flexible | Verbose scripting | Candidate |
| WiX | Proper MSI | Steep learning curve | Enterprise later |
| MSIX | Store-friendly | Constraints, different update model | Future evaluation |

Script: [`installer/inno/WinServeAI.iss`](../installer/inno/WinServeAI.iss). Stage
payload with [`scripts/stage-release.ps1`](../scripts/stage-release.ps1) into
`installer/inno/files/` (or `dist/WinServeAI`). Auto-updates are Phase 5+, not
installer MVP.

## Goals

* Install `winserve.exe`, `winserve-tray.exe`, and `bin/llama-server.exe` (+ private DLLs)
* Desktop + Start Menu shortcuts launch **tray** when present, else **manager CLI** — never `llama-server.exe` alone
* Default config at `config/default.yaml` (localhost bind; empty `model.path`)
* Ship `FIRST_RUN.txt` (Inno `InfoAfterFile` + `{app}\FIRST_RUN.txt`)
* Firewall rule only when optional “Allow LAN access” task is checked; remove on uninstall
* Ship `notices/` (THIRD_PARTY_NOTICES + llama.cpp LICENSE/AUTHORS + CUDA note)

## Layout on disk

```text
%ProgramFiles%\WinServeAI\          # {app}
  winserve.exe                      # CLI manager (serve / attach)
  winserve-tray.exe                 # primary shortcut target
  FIRST_RUN.txt                     # post-install guidance
  bin\
    llama-server.exe                # pinned llama.cpp build
    cudart64_XX.dll                 # CUDA builds only; beside the exe
    cublas*.dll                     # if linked (NVIDIA Attachment A only)
  config\
    default.yaml                    # empty model.path until operator sets .gguf
  logs\                             # created at runtime
  notices\
    THIRD_PARTY_NOTICES.md
    llama.cpp-LICENSE.txt
    llama.cpp-AUTHORS.txt
    NVIDIA-CUDA-NOTICE.txt          # CUDA builds
```

| Path | Role |
| --- | --- |
| `{app}\winserve-tray.exe` | Preferred entry; embeds `ServerManager`, holds lockfile + IPC |
| `{app}\winserve.exe` | CLI: `serve` / `status` / `stop` / `restart` / one-shot `start` |
| `{app}\bin\` | Backend binary and private DLLs; no PATH mutation |
| `{app}\config\` | Human-readable YAML defaults (`onlyifdoesntexist` on upgrade) |
| `{app}\FIRST_RUN.txt` | Short operator checklist (no weights bundled) |
| `{app}\logs\` | Unified server / llama / error logs |
| `{app}\notices\` | License compliance payload |

Inno icons prefer `{app}\winserve-tray.exe` with `WorkingDir: "{app}"`, falling
back to `winserve.exe` if the tray was not staged. Optional desktop icon via
`[Tasks]`. Admin elevation is required for Program Files and the optional
firewall task.

Do not put `llama-server.exe` on the Start Menu as the primary entry.

### After install — first run

1. Read `FIRST_RUN.txt` (also shown after Setup via `InfoAfterFile`).
2. Launch tray → Settings → Browse for a local `.gguf` → Save.
3. Start → wait for **Ready** → client at `http://127.0.0.1:8080/v1`.
4. CLI attach while tray is up: `winserve status|stop|restart`.

Details: [first-run.md](./first-run.md), [development.md](./development.md)
(lockfile / pipe paths).

## Firewall

Default bind is loopback (`127.0.0.1` / `localhost`). **Do not** add a firewall rule for loopback — Windows does not filter inbound loopback that way, and a rule implies LAN readiness that is not true.

| Bind | Firewall rule? |
| --- | --- |
| `127.0.0.1` / `localhost` | No |
| `0.0.0.0` / LAN interface | Yes — inbound allow for `{app}\bin\llama-server.exe` |

Prefer a **program-based** rule so port changes in YAML do not orphan rules:

```text
netsh advfirewall firewall add rule name="WinServeAI llama-server" dir=in action=allow program="{app}\bin\llama-server.exe" enable=yes profile=any
```

Uninstall must delete the same named rule. Wire via Inno `[Run]` / `[UninstallRun]` (`netsh.exe`, `runhidden`). Optional installer task “Allow LAN access” defaults **off**.

If the user changes bind after install, Server Manager should add or remove the rule so runtime stays in sync with config.

## Backend and CUDA DLLs

Ship `llama-server.exe` and any required CUDA runtime DLLs in the **same** `bin\` directory. Do not rely on a system CUDA Toolkit install or `PATH`. Pin the llama.cpp release used to build; redistribute only NVIDIA Attachment A libraries. End users still need a compatible NVIDIA driver.

Models are not shipped; GGUF files have separate licenses. Record the shipped
`b####` pin in release docs (I4).

## THIRD_PARTY_NOTICES

llama.cpp is MIT. Binary distributions must include the copyright and permission notice — a monorepo `LICENSE` alone is not enough ([ollama#3185](https://github.com/ollama/ollama/issues/3185)).

Minimum in the install tree:

* Rollup `notices/THIRD_PARTY_NOTICES.md` (component, upstream URL, license, pinned version/tag)
* Full llama.cpp `LICENSE` text
* Prefer `AUTHORS` from the pinned tag
* Notices for other bundled OSS and, for CUDA builds, an NVIDIA CUDA note

Copy notices into every installer and portable/zip release. Refresh with
`scripts/refresh-notices.sh` when the pin changes.

## Stage + compile

```powershell
.\scripts\stage-release.ps1 -OutDir installer\inno\files -RequireLlama
# then ISCC.exe installer\inno\WinServeAI.iss
```

## Phase 3 success

1. User runs the installer.
2. Desktop or Start Menu shortcut starts **winserve-tray** (or winserve fallback).
3. Operator sets `model.path` (Browse or YAML) — no weights in the payload.
4. Server Manager spawns `bin/llama-server.exe`, waits for readiness, exposes `/v1`.
5. Stopping / quitting the tray stops the backend and frees the GPU; CLI can
   `winserve stop` against the same owner.
6. Uninstall removes files and any firewall rule the installer created.

## Sources / See also

### Internal

- [research/04-installer-licensing.md](./research/04-installer-licensing.md) — Inno, firewall, CUDA DLLs, MIT notices
- [first-run.md](./first-run.md) — model path guidance
- [development.md](./development.md) — tray + IPC operator paths
- [architecture.md](./architecture.md) — resident owner
- [prior-art.md](./prior-art.md) — product analogues and installer patterns
- [release-process.md](./release-process.md) — pin + notices checklist
- [configuration.md](./configuration.md) — localhost vs LAN bind
- [backend.md](./backend.md) — `bin/` layout and DLL co-location
- [installer/README.md](../installer/README.md) — stub goals
- [bin/README.md](../bin/README.md) — external binary pin
- [apps/desktop/README.md](../apps/desktop/README.md) — tray shell

### Upstream

- [Inno Setup `[Icons]`](https://jrsoftware.org/ishelp/topic_iconssection.htm)
- [Inno admin install mode](https://jrsoftware.org/ishelp/topic_admininstallmode.htm)
- [Netsh AdvFirewall](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-r2-and-2008/dd734783(v=ws.10))
- [CUDA Toolkit EULA (Attachment A)](https://docs.nvidia.com/cuda/eula/) — redistributable DLLs only
- [llama.cpp LICENSE](https://github.com/ggml-org/llama.cpp/blob/master/LICENSE) (MIT)
- [ollama#3185](https://github.com/ollama/ollama/issues/3185) — binary distributions need notices

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
