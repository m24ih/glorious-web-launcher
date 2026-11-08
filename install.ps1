# Glorious Web Launcher - Windows Installation Script
[CmdletBinding()]
param(
    [switch]$Uninstall,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$Root = $PSScriptRoot
if (-not $Root) {
    $Root = (Get-Location).Path
}

$InstallDir = "$env:LOCALAPPDATA\glorious-launcher"
$StartMenuShortcut = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Glorious Web Launcher.lnk"
$StartupShortcut = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Glorious Launcher Autostart.lnk"

if ($Uninstall) {
    Write-Host "Uninstalling Glorious Web Launcher..." -ForegroundColor Yellow

    # Stop running process if any
    Get-Process -Name "glorious-service" -ErrorAction SilentlyContinue | Stop-Process -Force

    # Remove Start Menu shortcut
    if (Test-Path $StartMenuShortcut) {
        Write-Host "Removing Start Menu shortcut..."
        Remove-Item $StartMenuShortcut -Force
    }

    # Remove Startup shortcut
    if (Test-Path $StartupShortcut) {
        Write-Host "Removing Startup shortcut..."
        Remove-Item $StartupShortcut -Force
    }

    # Remove installation directory
    if (Test-Path $InstallDir) {
        Write-Host "Removing $InstallDir..."
        Remove-Item $InstallDir -Recurse -Force
    }

    Write-Host "Uninstallation completed successfully." -ForegroundColor Green
    return
}

Write-Host "=== Installing Glorious Web Launcher for Windows ===" -ForegroundColor Cyan

# 1. Check prerequisites
$PathEnv = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
$env:Path = $PathEnv

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Cargo (Rust) is required but not found in PATH."
}

if (-not $SkipBuild) {
    Write-Host "`n[1/4] Building frontend..." -ForegroundColor Yellow
    Push-Location "$Root\frontend"
    try {
        cmd.exe /c "npm ci && npm run build"
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }

    Write-Host "`n[2/4] Building release binary..." -ForegroundColor Yellow
    Push-Location $Root
    try {
        & cargo build --release -p glorious-service
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo release build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}

$ReleaseExe = "$Root\target\release\glorious-service.exe"
if (-not (Test-Path $ReleaseExe)) {
    throw "Release executable not found at $ReleaseExe"
}

Write-Host "`n[3/4] Installing application files to $InstallDir..." -ForegroundColor Yellow
if (-not (Test-Path "$InstallDir\data")) {
    New-Item -ItemType Directory -Path "$InstallDir\data" -Force | Out-Null
}

Copy-Item $ReleaseExe -Destination "$InstallDir\glorious-service.exe" -Force
Copy-Item "$Root\data\glorious_devices.json" -Destination "$InstallDir\data\glorious_devices.json" -Force

if (Test-Path "$Root\frontend\public\favicon.svg") {
    Copy-Item "$Root\frontend\public\favicon.svg" -Destination "$InstallDir\icon.svg" -Force
}

# 4. Create Start Menu Shortcut
Write-Host "`n[4/4] Creating shortcuts and background autostart task..." -ForegroundColor Yellow
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($StartMenuShortcut)
$Shortcut.TargetPath = "$InstallDir\glorious-service.exe"
$Shortcut.Arguments = "--open"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Configure Glorious mice in your browser"
$Shortcut.Save()

# 5. Create silent runner script for background startup without console window
$VbsScript = @"
Set WshShell = CreateObject("WScript.Shell")
Set FSO = CreateObject("Scripting.FileSystemObject")
If WScript.Arguments.Count > 0 Then
    ExePath = WScript.Arguments(0)
    AppDir = FSO.GetParentFolderName(ExePath)
    WshShell.CurrentDirectory = AppDir
    WshShell.Run """" & ExePath & """ --no-browser", 0, False
End If
"@
Set-Content -Path "$InstallDir\run-hidden.vbs" -Value $VbsScript -Encoding ASCII

$Startup = $WshShell.CreateShortcut($StartupShortcut)
$Startup.TargetPath = "wscript.exe"
$Startup.Arguments = "`"$InstallDir\run-hidden.vbs`" `"$InstallDir\glorious-service.exe`""
$Startup.WorkingDirectory = $InstallDir
$Startup.Description = "Glorious Web Launcher Background Service"
$Startup.Save()

# Stop existing running instance if any, then start background service
Get-Process -Name "glorious-service" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Process -FilePath "wscript.exe" -ArgumentList "`"$InstallDir\run-hidden.vbs`" `"$InstallDir\glorious-service.exe`"" -WorkingDirectory $InstallDir

Write-Host "`nInstallation completed successfully!" -ForegroundColor Green
Write-Host "Service is running in the background at: http://127.0.0.1:25820"
Write-Host "You can open it anytime from your Start Menu: 'Glorious Web Launcher'."
