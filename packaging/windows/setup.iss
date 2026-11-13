; Inno Setup Script for Glorious Web Launcher
; Produces a standalone, single-file installer for Windows users without any dependencies.

#define MyAppName "Glorious Web Launcher"
#define MyAppVersion "0.1.9"
#define MyAppPublisher "m24ih"
#define MyAppURL "https://gitlab.com/m24ih/glorious-web-launcher"
#define MyAppExeName "glorious-service.exe"

[Setup]
; Unique GUID identifying this application
AppId={{B74D1C36-239C-4E25-8BFA-5E5D57AC0580}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases

; Install to Local AppData so standard users do NOT need administrator / UAC elevation
DefaultDirName={localappdata}\GloriousLauncher
DisableDirPage=no
DisableProgramGroupPage=yes
PrivilegesRequired=lowest

; Output settings
OutputDir=..\..\target\installer
OutputBaseFilename=GloriousWebLauncher-Setup-v{#MyAppVersion}
SetupIconFile=app.ico
UninstallDisplayIcon={app}\app.ico

; Modern look & high-ratio compression
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern

; Prevent running multiple installer instances
SetupMutex=GloriousWebLauncherSetupMutex
UsePreviousTasks=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "turkish"; MessagesFile: "compiler:Languages\Turkish.isl"

[CustomMessages]
english.AutostartGroup=Startup Options:
english.AutostartDescription=Start Glorious Web Launcher automatically with Windows (System Tray)
turkish.AutostartGroup=Başlangıç Seçenekleri:
turkish.AutostartDescription=Glorious Web Launcher'ı Windows açılışında otomatik başlat (Sistem Tepsisi)

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "{cm:AutostartDescription}"; GroupDescription: "{cm:AutostartGroup}"

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\data\glorious_devices.json"; DestDir: "{app}\data"; Flags: ignoreversion
Source: "app.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "run-hidden.vbs"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Start Menu
Name: "{autoprograms}\{#MyAppName}"; Filename: "wscript.exe"; Parameters: """{app}\run-hidden.vbs"" ""{app}\{#MyAppExeName}"" --open"; IconFilename: "{app}\app.ico"; WorkingDir: "{app}"
; Desktop shortcut (optional)
Name: "{autodesktop}\{#MyAppName}"; Filename: "wscript.exe"; Parameters: """{app}\run-hidden.vbs"" ""{app}\{#MyAppExeName}"" --open"; IconFilename: "{app}\app.ico"; WorkingDir: "{app}"; Tasks: desktopicon
; Autostart on Windows Login (background tray without opening browser)
Name: "{userstartup}\{#MyAppName}"; Filename: "wscript.exe"; Parameters: """{app}\run-hidden.vbs"" ""{app}\{#MyAppExeName}"" --no-browser"; IconFilename: "{app}\app.ico"; WorkingDir: "{app}"; Tasks: autostart

[Run]
; Launch silently to system tray and open browser UI when finished
Filename: "wscript.exe"; Parameters: """{app}\run-hidden.vbs"" ""{app}\{#MyAppExeName}"" --open"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait

[UninstallRun]
; Kill running service before uninstalling files
Filename: "taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden; RunOnceId: "StopGloriousService"

[Code]
// Custom Inno Pascal script: Clean up running instances before new install
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  // Terminate any currently running instance of glorious-service before replacing files
  Exec('taskkill.exe', '/F /IM ' + '{#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := '';
end;
