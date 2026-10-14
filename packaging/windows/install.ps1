# FocusFort installer for Windows (run as Administrator).
# Copies the agent, registers it as an automatic Windows service and opens the app.
#Requires -RunAsAdministrator
param(
    [string]$Source = $PSScriptRoot,
    [string]$InstallDir = "$env:ProgramFiles\FocusFort"
)
$ErrorActionPreference = "Stop"
$service = "FocusFort"
$exe = Join-Path $InstallDir "focusfort-agent.exe"

if (Get-Service -Name $service -ErrorAction SilentlyContinue) {
    Stop-Service -Name $service -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item (Join-Path $Source "focusfort-agent.exe") $exe -Force
if (Test-Path (Join-Path $Source "extension")) {
    Copy-Item (Join-Path $Source "extension") (Join-Path $InstallDir "extension") -Recurse -Force
}
Copy-Item (Join-Path $Source "uninstall.ps1") (Join-Path $InstallDir "uninstall.ps1") -Force

if (-not (Get-Service -Name $service -ErrorAction SilentlyContinue)) {
    New-Service -Name $service -BinaryPathName "`"$exe`" service" -DisplayName "FocusFort" `
        -Description "FocusFort website and app blocker" -StartupType Automatic | Out-Null
}
# Restart the service automatically if it ever stops unexpectedly.
sc.exe failure $service reset= 86400 actions= restart/5000/restart/5000/restart/30000 | Out-Null
Start-Service -Name $service

# Start menu shortcut to the local UI.
$shortcut = Join-Path ([Environment]::GetFolderPath("CommonStartMenu")) "Programs\FocusFort.url"
Set-Content -Path $shortcut -Value "[InternetShortcut]`r`nURL=http://127.0.0.1:17345/`r`nIconFile=$exe`r`nIconIndex=0" -Encoding ASCII

# Add/Remove Programs entry that runs our uninstaller (which honours uninstall protection).
$key = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\FocusFort"
New-Item -Path $key -Force | Out-Null
Set-ItemProperty $key DisplayName "FocusFort"
Set-ItemProperty $key Publisher "FocusFort"
Set-ItemProperty $key DisplayVersion "0.1.0"
Set-ItemProperty $key DisplayIcon $exe
Set-ItemProperty $key InstallLocation $InstallDir
Set-ItemProperty $key UninstallString "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall.ps1`""
Set-ItemProperty $key NoModify 1
Set-ItemProperty $key NoRepair 1

Start-Process "http://127.0.0.1:17345/"
Write-Host "FocusFort installed."
