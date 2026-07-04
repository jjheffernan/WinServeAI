# Installer

Native Windows installer (Inno Setup preferred).

## Goals

* Install `winserve.exe` + `bin/llama-server.exe`
* Desktop shortcut → starts Server Manager
* Start menu entry
* Firewall rule only when bind is non-loopback
* `config/default.yaml` + `THIRD_PARTY_NOTICES`

## Layout on disk (installed)

```text
%ProgramFiles%\WinServeAI\
  winserve.exe
  bin\llama-server.exe
  config\default.yaml
  logs\
  THIRD_PARTY_NOTICES.txt
```
