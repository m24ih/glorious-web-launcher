<#
.SYNOPSIS
    Builds the standalone Windows GUI Installer (.exe) for Glorious Web Launcher.

.DESCRIPTION
    Compiles frontend (Vite), backend (Cargo release binary), and packages everything
    into a single-file, zero-dependency "Next-Next-Next" Inno Setup installer.
    
    The resulting installer requires NO Rust, Cargo, or Node.js on end-user machines.

.PARAMETER SkipFrontend
    Skip building the frontend if already built.

.PARAMETER SkipCargo
    Skip building the Rust binary if already compiled.
#>

[CmdletBinding()]
param(
    [switch]$SkipFrontend,
    [switch]$SkipCargo
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

Write-Host "======================================================" -ForegroundColor Cyan
Write-Host "  Glorious Web Launcher - Windows Installer Builder  " -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan

# 1. Locate Inno Setup Compiler (ISCC.exe)
$Candidates = @(
    (Get-Command iscc.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source),
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
)

$IsccPath = $null
foreach ($c in $Candidates) {
    if ($c -and (Test-Path $c)) {
        $IsccPath = $c
        break
    }
}

if (-not $IsccPath) {
    throw "Inno Setup Compiler (ISCC.exe) was not found. Please install it with: winget install JRSoftware.InnoSetup -e"
}
Write-Host "[+] Found Inno Setup Compiler: $IsccPath" -ForegroundColor Green

# 2. Build Frontend
if (-not $SkipFrontend) {
    Write-Host "`n[1/3] Building frontend assets with Vite..." -ForegroundColor Yellow
    Push-Location "$RootDir\frontend"
    try {
        cmd.exe /c "npm ci && npm run build"
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
} else {
    Write-Host "`n[1/3] Skipping frontend build as requested." -ForegroundColor DarkGray
}

# 3. Build Cargo Release Binary
if (-not $SkipCargo) {
    Write-Host "`n[2/3] Building Rust release binary..." -ForegroundColor Yellow
    Push-Location $RootDir
    try {
        & cargo build --release -p glorious-service
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo release build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
} else {
    Write-Host "`n[2/3] Skipping Cargo build as requested." -ForegroundColor DarkGray
}

$ReleaseExe = "$RootDir\target\release\glorious-service.exe"
if (-not (Test-Path $ReleaseExe)) {
    throw "Release binary not found at $ReleaseExe"
}

# 4. Compile Inno Setup Script
Write-Host "`n[3/3] Compiling Inno Setup package..." -ForegroundColor Yellow
$IssPath = "$RootDir\packaging\windows\setup.iss"
Push-Location (Split-Path -Parent $IssPath)
try {
    & $IsccPath $IssPath
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup compilation failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

$Installer = Get-ChildItem -Path "$RootDir\target\installer" -Filter "GloriousWebLauncher-Setup*.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$InstallerPath = if ($Installer) { $Installer.FullName } else { "" }
if (Test-Path $InstallerPath) {
    $Item = Get-Item $InstallerPath
    $SizeMb = [math]::Round($Item.Length / 1MB, 2)
    $Hash = (Get-FileHash -Path $InstallerPath -Algorithm SHA256).Hash
    
    Write-Host "`n======================================================" -ForegroundColor Green
    Write-Host " Installer created successfully!" -ForegroundColor Green
    Write-Host " Location : $InstallerPath" -ForegroundColor Green
    Write-Host " Size     : $SizeMb MB" -ForegroundColor Green
    Write-Host " SHA-256  : $Hash" -ForegroundColor Green
    Write-Host "======================================================" -ForegroundColor Green
} else {
    throw "Installer binary not found at $InstallerPath"
}
