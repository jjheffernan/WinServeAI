# Addons

Frontends are not part of `ServerManager`. An addon is any client of `http://host:port/v1`.

To connect one you need two values from WinServeAI:

| Value | Where |
| --- | --- |
| Base URL | Tray “Copy URL”, or `winserve tui` view `3`. Default `http://127.0.0.1:8080/v1` |
| API key | `server.api_key` in `config/default.yaml`, or Advanced in the tray. Empty on this PC. |

Send the key as `Authorization: Bearer <key>`. Leave the header off when the key is empty.

`chat/` is the bundled example. Replace that folder, or point another app (Cursor, SillyTavern, your own page) at the same URL and key. The server does not load, theme, or route addon code.
