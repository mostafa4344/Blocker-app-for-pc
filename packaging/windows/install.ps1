# ContentGuard installer for Windows (run as Administrator).
# Copies the agent, registers it as an automatic Windows service and opens the app.
#Requires -RunAsAdministrator
param(
    [string]$Source = $PSScriptRoot,
    [string]$InstallDir = "$env:ProgramFiles\ContentGuard"
)
$ErrorActionPreference = "Stop"
$service = "ContentGuard"
$exe = Join-Path $InstallDir "contentguard-agent.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
# While protection is on the running agent refuses to stop, so upgrade in place:
# move the running exe aside (allowed on Windows), then copy the new one in.
$old = "$exe.old"
Remove-Item $old -Force -ErrorAction SilentlyContinue
if (Test-Path $exe) { Move-Item $exe $old -Force }
Copy-Item (Join-Path $Source "contentguard-agent.exe") $exe -Force
if (Test-Path (Join-Path $Source "extension")) {
    Copy-Item (Join-Path $Source "extension") (Join-Path $InstallDir "extension") -Recurse -Force
}
Copy-Item (Join-Path $Source "uninstall.ps1") (Join-Path $InstallDir "uninstall.ps1") -Force

if (-not (Get-Service -Name $service -ErrorAction SilentlyContinue)) {
    New-Service -Name $service -BinaryPathName "`"$exe`" service" -DisplayName "ContentGuard" `
        -Description "ContentGuard website and app blocker" -StartupType Automatic | Out-Null
}
sc.exe config $service binPath= "`"$exe`" service" start= auto | Out-Null
# Restart automatically if the process ever ends (crash or forced kill).
sc.exe failure $service reset= 86400 actions= restart/3000/restart/3000/restart/10000 | Out-Null
sc.exe failureflag $service 1 | Out-Null
# Restart into the new exe (normal stop when allowed, otherwise end the process
# and let the recovery action above start the new one).
Stop-Service -Name $service -Force -ErrorAction SilentlyContinue
taskkill /F /IM contentguard-agent.exe 2>$null | Out-Null
Start-Sleep -Seconds 2
Start-Service -Name $service -ErrorAction SilentlyContinue
Remove-Item $old -Force -ErrorAction SilentlyContinue

# Start menu shortcut to the local UI.
$shortcut = Join-Path ([Environment]::GetFolderPath("CommonStartMenu")) "Programs\ContentGuard.url"
Set-Content -Path $shortcut -Value "[InternetShortcut]`r`nURL=http://127.0.0.1:17345/`r`nIconFile=$exe`r`nIconIndex=0" -Encoding ASCII

# Add/Remove Programs entry that runs our uninstaller (which honours uninstall protection).
$key = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ContentGuard"
New-Item -Path $key -Force | Out-Null
Set-ItemProperty $key DisplayName "ContentGuard"
Set-ItemProperty $key Publisher "ContentGuard"
Set-ItemProperty $key DisplayVersion "0.1.0"
Set-ItemProperty $key DisplayIcon $exe
Set-ItemProperty $key InstallLocation $InstallDir
Set-ItemProperty $key UninstallString "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall.ps1`""
Set-ItemProperty $key NoModify 1
Set-ItemProperty $key NoRepair 1

Start-Process "http://127.0.0.1:17345/"
Write-Host "ContentGuard installed."
