# Installer

## Goals

* Desktop shortcut
* Start menu entry
* Firewall rules
* Bundled backend
* Auto updates (future)

## Research Checklist

| Option | Pros | Cons | Status |
| --- | --- | --- | --- |
| Inno Setup | Simple, scriptable, popular for desktop apps | Not MSI-native | **Preferred starting point** |
| NSIS | Mature, flexible | Verbose scripting | Candidate |
| WiX | Proper MSI | Steep learning curve | Candidate for enterprise |
| MSIX | Modern, store-friendly | Constraints, update model differs | Future evaluation |

## Phase 3 Deliverable

Install → Start → OpenAI-compatible API available on the configured host/port.

## Firewall

* Default LAN bind (`0.0.0.0`) may require an inbound rule
* Localhost mode (`127.0.0.1`) should not open the firewall
* Rules must be reversible on uninstall
