<#
.SYNOPSIS
    Installer for BetterFolderSize (v0.1 MVP)
.DESCRIPTION
    Copies the binary to %LOCALAPPDATA%\BetterFolderSize\
    and registers the context menu entry in Windows Explorer.
#>

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Installing BetterFolderSize v0.1    " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$InstallDir = Join-Path $env:LOCALAPPDATA "BetterFolderSize"
$TargetExe = Join-Path $InstallDir "BetterFolderSize.exe"

# 1. Locate the executable binary
$SourceExe = $null
$Candidates = @(
    (Join-Path $ScriptDir "BetterFolderSize.exe"),
    (Join-Path $ScriptDir "better-folder-size.exe"),
    (Join-Path $ProjectRoot "target\release\better-folder-size.exe"),
    (Join-Path $ProjectRoot "target\release\BetterFolderSize.exe")
)

foreach ($c in $Candidates) {
    if (Test-Path $c) {
        $SourceExe = $c
        break
    }
}

if (-not $SourceExe) {
    Write-Host "[*] Release binary not found. Building project with cargo..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    try {
        & cargo build --release
        $SourceExe = Join-Path $ProjectRoot "target\release\better-folder-size.exe"
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path $SourceExe)) {
    Write-Error "Failed to find or build BetterFolderSize.exe."
}

# 2. Create destination directory and copy executable
Write-Host "[*] Creating target directory: $InstallDir" -ForegroundColor Green
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "[*] Copying binary..." -ForegroundColor Green
Copy-Item -Path $SourceExe -Destination $TargetExe -Force

# 3. Register Explorer context menu in Windows Registry (HKCU - no Admin rights required)
Write-Host "[*] Registering Windows Explorer context menu..." -ForegroundColor Green

$MenuName = "Folder Size (Better)"
$CommandStr = "`"$TargetExe`" `"%1`""
$CommandBgStr = "`"$TargetExe`" `"%V`""

# Context menu for Folders (Directory)
$RegDirShell = "HKCU:\Software\Classes\Directory\shell\BetterFolderSize"
$RegDirCmd = "$RegDirShell\command"
New-Item -Path $RegDirShell -Force | Out-Null
Set-ItemProperty -Path $RegDirShell -Name "(default)" -Value $MenuName
New-Item -Path $RegDirCmd -Force | Out-Null
Set-ItemProperty -Path $RegDirCmd -Name "(default)" -Value $CommandStr

# Context menu for Folder Background (Directory\Background)
$RegBgShell = "HKCU:\Software\Classes\Directory\Background\shell\BetterFolderSize"
$RegBgCmd = "$RegBgShell\command"
New-Item -Path $RegBgShell -Force | Out-Null
Set-ItemProperty -Path $RegBgShell -Name "(default)" -Value $MenuName
New-Item -Path $RegBgCmd -Force | Out-Null
Set-ItemProperty -Path $RegBgCmd -Name "(default)" -Value $CommandBgStr

# Context menu for Drives (Drive)
$RegDriveShell = "HKCU:\Software\Classes\Drive\shell\BetterFolderSize"
$RegDriveCmd = "$RegDriveShell\command"
New-Item -Path $RegDriveShell -Force | Out-Null
Set-ItemProperty -Path $RegDriveShell -Name "(default)" -Value $MenuName
New-Item -Path $RegDriveCmd -Force | Out-Null
Set-ItemProperty -Path $RegDriveCmd -Name "(default)" -Value $CommandStr

Write-Host ""
Write-Host "[OK] Installation completed successfully!" -ForegroundColor Green
Write-Host "     You can now right-click any folder in Windows Explorer"
Write-Host "     and choose '$MenuName'."
Write-Host ""
