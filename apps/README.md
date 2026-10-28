# Apps

People open these. The server program lives in `app/` and is not one of them.

| Folder | What it is |
| --- | --- |
| `tray/` | The window. Browse for a model, Start, Copy URL. |
| `chat/` | A chat page. It only calls `/v1`. Swap this folder for another frontend. |

To connect any frontend you need two values:

| Value | Where |
| --- | --- |
| Base URL | **Copy URL** in the window, or `winserve tui` view `3`. Default `http://127.0.0.1:8080/v1` |
| API key | Advanced in the window, or `server.api_key` in `config/default.yaml`. Leave it empty on this PC. |

Send the key as `Authorization: Bearer <key>`. Leave the header off when the key is empty. The server does not load frontend code.
