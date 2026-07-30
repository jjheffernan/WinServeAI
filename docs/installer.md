# Installer

> **Readiness:** installer 2.2/5 (`scaffold`), bin 2.2/5 (`scaffold`) — details in [readiness/installer.md](./readiness/installer.md), [readiness/bin.md](./readiness/bin.md)

Native Windows packaging for the WinServeAI appliance. Phase 3 deliverable: install → launch **winserve** → OpenAI-compatible API on the configured host/port.

Deep research: [research/04-installer-licensing.md](./research/04-installer-licensing.md), [research/doc-build/installer.research.md](./research/doc-build/installer.research.md). Build spec (Phase 3, before Inno code): [specs/F-installer.md](./specs/F-installer.md). Stub goals: [installer/README.md](../installer/README.md). Prior art: [prior-art.md](./prior-art.md).

## Tool choice

| Option | Pros | Cons | Status |
| --- | --- | --- | --- |
| **Inno Setup** | Simple, scriptable, common for desktop apps | Not MSI-native | **Preferred starting point** |
| NSIS | Mature, flexible | Verbose scripting | Candidate |
| WiX | Proper MSI | Steep learning curve | Enterprise later |
| MSIX | Store-friendly | Constraints, different update model | Future evaluation |

Scripts live under `installer/`. Prefer Inno Setup for MVP. Auto-updates are Phase 5+, not installer MVP.

## Goals

* Install `winserve.exe` and `bin/llama-server.exe` (plus private DLLs)
* Desktop shortcut and Start Menu entry launch **`winserve`** (Server Manager), not `llama-server.exe`
* Default config at `config/default.yaml` (localhost bind)
* Firewall rule only when bind is non-loopback; remove on uninstall
* Ship `THIRD_PARTY_NOTICES` (and full license texts) in the install payload

## Layout on disk

```text
%ProgramFiles%\WinServeAI\          # {app}
  winserve.exe                      # Server Manager — shortcut target
  bin\
    llama-server.exe                # pinned llama.cpp build
    cudart64_XX.dll                 # CUDA builds only; beside the exe
    cublas*.dll                     # if linked (NVIDIA Attachment A only)
  config\
    default.yaml
  logs\                             # created at runtime
  notices\
    THIRD_PARTY_NOTICES.md
    llama.cpp-LICENSE.txt
    llama.cpp-AUTHORS.txt
    NVIDIA-CUDA-NOTICE.txt          # CUDA builds
```

| Path | Role |
| --- | --- |
| `{app}\winserve.exe` | Sole primary entry point; process owner for `llama-server.exe` |
| `{app}\bin\` | Backend binary and private DLLs; no PATH mutation |
| `{app}\config\` | Human-readable YAML defaults |
| `{app}\logs\` | Unified server / llama / error logs |
| `{app}\notices\` | License compliance payload |

Inno icons use `Filename: "{app}\winserve.exe"` and `WorkingDir: "{app}"`. Optional desktop icon via a `[Tasks]` checkbox. Include a standard Uninstall entry. Admin elevation is required for Program Files and firewall rules.

Do not put `llama-server.exe` on the Start Menu as the primary entry.

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

Models are not shipped; GGUF files have separate licenses.

## THIRD_PARTY_NOTICES

llama.cpp is MIT. Binary distributions must include the copyright and permission notice — a monorepo `LICENSE` alone is not enough ([ollama#3185](https://github.com/ollama/ollama/issues/3185)).

Minimum in the install tree:

* Rollup `notices/THIRD_PARTY_NOTICES.md` (component, upstream URL, license, pinned version/tag)
* Full llama.cpp `LICENSE` text
* Prefer `AUTHORS` from the pinned tag
* Notices for other bundled OSS and, for CUDA builds, an NVIDIA CUDA note

Copy notices into every installer and portable/zip release. Refresh when the `vendor/llama.cpp` pin changes.

## Phase 3 success

1. User runs the installer.
2. Desktop or Start Menu shortcut starts **winserve**.
3. Server Manager spawns `bin/llama-server.exe`, waits for readiness (`GET /v1/models`), exposes `/v1`.
4. Stopping winserve stops the backend and frees the GPU.
5. Uninstall removes files and any firewall rule the installer (or manager) created.

## Sources / See also

### Internal

- [research/04-installer-licensing.md](./research/04-installer-licensing.md) — Inno, firewall, CUDA DLLs, MIT notices
- [prior-art.md](./prior-art.md) — product analogues and installer patterns
- [release-process.md](./release-process.md) — pin + notices checklist
- [configuration.md](./configuration.md) — localhost vs LAN bind
- [backend.md](./backend.md) — `bin/` layout and DLL co-location
- [installer/README.md](../installer/README.md) — stub goals
- [bin/README.md](../bin/README.md) — external binary pin

### Upstream

- [Inno Setup `[Icons]`](https://jrsoftware.org/ishelp/topic_iconssection.htm)
- [Inno admin install mode](https://jrsoftware.org/ishelp/topic_admininstallmode.htm)
- [Netsh AdvFirewall](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-r2-and-2008/dd734783(v=ws.10))
- [CUDA Toolkit EULA (Attachment A)](https://docs.nvidia.com/cuda/eula/) — redistributable DLLs only
- [llama.cpp LICENSE](https://github.com/ggml-org/llama.cpp/blob/master/LICENSE) (MIT)
- [ollama#3185](https://github.com/ollama/ollama/issues/3185) — binary distributions need notices

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
