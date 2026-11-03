# Sandbox launch checklist

Run this on a clean Windows 10 or 11 virtual machine. Take a snapshot before step 1. There is no published GitHub release yet, so the setup has to be built on a Windows machine that has the repo, then copied in.

Record each line as pass or fail. A fail with the note in the last column is a known hole, not a surprise.

## Bring onto the VM

Copy only these. Do not copy `target\` or a half-built `bin\`.

- `dist\WinServeAI-0.1.0-setup.exe`
- One small `.gguf` you already have (a few GB is enough; the readiness wait is 120 seconds)
- This file

Build the setup on the other Windows machine, from the repo root, in a Visual Studio developer shell (`where.exe link` must find `link.exe`):

```powershell
.\scripts\fetch-llama-pin.ps1
cargo build -p winserve --release
cargo build -p winserve-tray --release
.\scripts\stage-release.ps1 -Configuration release -SkipBuild -RequireLlama
choco install innosetup -y
& "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe" installer\inno\WinServeAI.iss
```

If Chocolatey is not installed, install Inno Setup 6 yourself and run that same `ISCC.exe` line. The compiler is `C:\Program Files (x86)\Inno Setup 6\ISCC.exe`.

`fetch-llama-pin.ps1` with no arguments downloads the Vulkan build, pin `b9866`. `-RequireLlama` refuses to stage if `bin\llama-server.exe` is missing. The setup lands at `dist\WinServeAI-0.1.0-setup.exe`.

## 1. Install

- [ ] Run the setup. Leave the per-user default (`%LOCALAPPDATA%\Programs\WinServeAI`). Do not check Allow LAN access.
- [ ] Start menu shortcut is WinServeAI. It is not `llama-server.exe`.
- [ ] `{app}\bin\llama-server.exe` exists.
- [ ] `{app}\config\default.yaml` has an empty `model.path`.
- [ ] `{app}\notices\` contains the llama.cpp license.

Pass: the window opens and says the model path is empty. Nothing is listening on port 8080 yet.

## 2. A missing model stays on screen

- [ ] Browse to a file that is not a `.gguf`, or type a path that does not exist. Save, then Start.
- [ ] Read the red line. Wait 5 seconds without clicking anything.

Pass today: Start refuses before spawn. The sentence names the path or the GGUF header.

Known hole: the 2-second refresh clears that sentence. If it vanishes, write `banner cleared` and continue. The tray menu Start item also does not show the sentence.

## 3. A taken port stays on screen

- [ ] In PowerShell: `python -m http.server 8080`
- [ ] In the window, Browse to the real `.gguf`, Save, Start.
- [ ] Read the red line. Wait 5 seconds.
- [ ] Stop the Python server (`Ctrl+C`).

Pass: Start fails before the model loads, and the line names port 8080.

Known hole: same as step 2. The line may disappear on the next poll.

## 4. Ready

- [ ] Start again with the Python server stopped.
- [ ] Wait until the badge says Ready. First load can take the whole 120 seconds.
- [ ] Copy connect. The clipboard has Base URL, API key, and model id. The URL is `http://127.0.0.1:8080/v1`.

```powershell
Invoke-RestMethod http://127.0.0.1:8080/v1/models
```

Pass: HTTP 200 and the model id is the `.gguf` path.

Then the common completion routes (or run `.\scripts\smoke-openai.ps1`):

```powershell
$model = (Invoke-RestMethod http://127.0.0.1:8080/v1/models).data[0].id
$oai = @{ model = $model; messages = @(@{ role = "user"; content = "Say hi" }); max_tokens = 16 } | ConvertTo-Json -Depth 5
Invoke-RestMethod http://127.0.0.1:8080/v1/chat/completions -Method Post -Body $oai -ContentType "application/json"
$rsp = @{ model = $model; input = "Say hi"; max_output_tokens = 16 } | ConvertTo-Json -Depth 5
Invoke-RestMethod http://127.0.0.1:8080/v1/responses -Method Post -Body $rsp -ContentType "application/json"
$ant = @{ model = $model; max_tokens = 16; messages = @(@{ role = "user"; content = "Say hi" }) } | ConvertTo-Json -Depth 5
Invoke-RestMethod http://127.0.0.1:8080/v1/messages -Method Post -Body $ant -ContentType "application/json"
$leg = @{ model = $model; prompt = "Say hi"; max_tokens = 16 } | ConvertTo-Json -Depth 5
Invoke-RestMethod http://127.0.0.1:8080/v1/completions -Method Post -Body $leg -ContentType "application/json"
```

Pass: a short reply on each. If any returns 500 and the llama log contains `out of memory` or `Compute error` while the badge still says Ready, write `GPU load failed but Ready`. That is a known hole. If `/v1/messages` or `/v1/responses` is 404, the llama.cpp pin is too old.

## 5. The window while it is serving

- [ ] Overview shows the model file, context, and one card per GPU. A card that holds the model shows the filename.
- [ ] Open chat. Send one line. Stop aborts the stream.
- [ ] Close the window with the X. The API still answers `/v1/models`.
- [ ] Open WinServeAI again. A second window does not stay open. The first one comes back.

Pass: X hides to the tray. Quit is the stop.

## 6. Quit frees the GPU

- [ ] Tray menu → Quit.
- [ ] `Get-Process llama-server` returns nothing.
- [ ] `Invoke-RestMethod http://127.0.0.1:8080/v1/models` fails.

Pass: no leftover `llama-server`. If Quit shows no error and the process remains, write `orphan`.

## 7. Reboot

- [ ] Start, wait for Ready, then restart the VM.
- [ ] After login, do not open the app. Run the `/v1/models` request again.

Pass today: the request fails. Nothing starts at boot. Write `down after reboot`. That is a hole, not a failed install.

## 8. Uninstall

- [ ] Settings → Apps → WinServeAI → Uninstall.
- [ ] `{app}` is gone.
- [ ] No firewall rule named `WinServeAI llama-server` (you never enabled LAN, so there should not have been one).

```powershell
netsh advfirewall firewall show rule name="WinServeAI llama-server"
```

Pass: `No rules match`.

## From the repo, if this VM is the build machine

Skip the setup and run these from the repo root after the Vulkan fetch and the two release builds. Point `model.path` at the `.gguf` first for the smoke script.

```powershell
.\tests\windows\a2-smoke.ps1 -Start
.\tests\windows\tray-quit.ps1 -StartTray
.\tests\windows\orphan-quit.ps1 -Start
```

`a2-smoke.ps1 -Start` leaves `winserve` running when it passes. Quit that process before the orphan script, or the port is still taken. `tray-quit.ps1 -StartTray` leaves the tray up. Use the tray Quit menu, then confirm `Get-Process llama-server` is empty.

## Sheet

| Step | Pass / fail | Note |
| --- | --- | --- |
| 1 Install contains llama-server | | |
| 2 Bad model sentence stays | | |
| 3 Taken port sentence stays | | |
| 4 `/v1/models` and one completion | | |
| 5 X hides, second window refused | | |
| 6 Quit leaves no llama-server | | |
| 7 Still down after reboot | | expected |
| 8 Uninstall is clean | | |
