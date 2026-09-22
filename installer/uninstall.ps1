<#
.SYNOPSIS
    Uninstaller for BetterFolderSize
.DESCRIPTION
    Removes context menu entries from registry and deletes %LOCALAPPDATA%\BetterFolderSize.
#>

$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "   Uninstalling BetterFolderSize v0.1   " -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host ""

# 1. Terminate any running instances
Write-Host "[*] Terminating running instances..." -ForegroundColor Cyan
Get-Process -Name "BetterFolderSize", "better-folder-size" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. Remove registry keys
$KeysToRemove = @(
    "HKCU:\Software\Classes\Directory\shell\BetterFolderSize",
    "HKCU:\Software\Classes\Directory\Background\shell\BetterFolderSize",
    "HKCU:\Software\Classes\Drive\shell\BetterFolderSize"
)

foreach ($key in $KeysToRemove) {
    if (Test-Path $key) {
        Write-Host "[*] Removing registry key: $key" -ForegroundColor Cyan
        Remove-Item -Path $key -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# 3. Remove installation directory
$InstallDir = Join-Path $env:LOCALAPPDATA "BetterFolderSize"
if (Test-Path $InstallDir) {
    Write-Host "[*] Removing installation directory: $InstallDir" -ForegroundColor Cyan
    Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "[OK] BetterFolderSize has been completely uninstalled." -ForegroundColor Green
Write-Host ""
