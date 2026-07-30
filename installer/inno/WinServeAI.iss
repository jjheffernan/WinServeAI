; WinServeAI — Inno Setup 6 script (F1–F3)
;
; Prerequisites:
;   1. Stage payload into installer\inno\files\ :
;        .\scripts\stage-release.ps1 -OutDir installer\inno\files -RequireLlama
;   2. Install Inno Setup 6+
;   3. Compile this script (ISCC.exe WinServeAI.iss)
;
; Shortcuts launch winserve-tray.exe when present, else winserve.exe — never llama-server alone.
; Firewall rule is opt-in (task "Allow LAN access", default off) for non-loopback binds.

#define MyAppName "WinServeAI"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "WinServeAI Contributors"
#define MyAppURL "https://github.com/jjheffernan/WinServeAI"
#define MyAppExeName "winserve.exe"
#define MyAppTrayName "winserve-tray.exe"
#define MyFirewallRuleName "WinServeAI llama-server"

[Setup]
AppId={{A8F3C2E1-9B47-4D6A-8E21-7C0F5B1D4A90}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
; Staged payload from scripts\stage-release.ps1
SourceDir=files
OutputDir=..\..\dist
OutputBaseFilename=WinServeAI-{#MyAppVersion}-setup
SetupIconFile=
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}
DisableProgramGroupPage=no
InfoAfterFile=

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "lanfirewall"; Description: "Allow LAN access (Windows Firewall inbound rule for llama-server.exe)"; GroupDescription: "Network:"; Flags: unchecked

[Files]
; Manager + tray (tray optional at stage time — use Check below)
Source: "winserve.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "winserve-tray.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
; Backend + private DLLs
Source: "bin\*"; DestDir: "{app}\bin"; Flags: ignoreversion recursesubdirs createallsubdirs
; Default config — do not clobber operator edits on upgrade
Source: "config\default.yaml"; DestDir: "{app}\config"; Flags: onlyifdoesntexist uninsneveruninstall
; Notices (MIT / third-party)
Source: "notices\*"; DestDir: "{app}\notices"; Flags: ignoreversion recursesubdirs createallsubdirs
; Empty logs dir
Source: "logs\*"; DestDir: "{app}\logs"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist

[Icons]
; Prefer tray when staged; fallback to CLI manager.
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppTrayName}"; WorkingDir: "{app}"; Check: TrayExists; Comment: "WinServeAI desktop shell"
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Check: not TrayExists; Comment: "WinServeAI manager (CLI)"
Name: "{group}\{#MyAppName} CLI"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Comment: "winserve CLI (serve/status/stop)"
Name: "{group}\Third-party notices"; Filename: "{app}\notices\THIRD_PARTY_NOTICES.md"; WorkingDir: "{app}\notices"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppTrayName}"; WorkingDir: "{app}"; Tasks: desktopicon; Check: TrayExists
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon; Check: not TrayExists

[Run]
; Optional LAN firewall — only when task selected (default bind is loopback; no rule needed then).
Filename: "{sys}\netsh.exe"; \
  Parameters: "advfirewall firewall add rule name=""{#MyFirewallRuleName}"" dir=in action=allow program=""{app}\bin\llama-server.exe"" enable=yes profile=any"; \
  Flags: runhidden; \
  Tasks: lanfirewall; \
  StatusMsg: "Adding Windows Firewall rule for LAN access…"

[UninstallRun]
Filename: "{sys}\netsh.exe"; \
  Parameters: "advfirewall firewall delete rule name=""{#MyFirewallRuleName}"""; \
  Flags: runhidden; \
  RunOnceId: "RemoveWinServeFirewall"

[Code]
function TrayExists: Boolean;
begin
  Result := FileExists(ExpandConstant('{app}\{#MyAppTrayName}'));
end;
