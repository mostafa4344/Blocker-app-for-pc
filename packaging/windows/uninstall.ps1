# ContentGuard uninstaller for Windows. Honours the uninstall protection chosen in the app:
# if protection is on, an approved uninstall request is required first.
param([string]$InstallDir = "$env:ProgramFiles\ContentGuard")
$ErrorActionPreference = "Stop"
$service = "ContentGuard"
$exe = Join-Path $InstallDir "contentguard-agent.exe"

# Re-launch elevated if needed.
$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) {
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -File `"$PSCommandPath`" -InstallDir `"$InstallDir`""
    exit
}

if (Test-Path $exe) {
    $out = & $exe uninstall-check
    if ($LASTEXITCODE -ne 0) {
        Add-Type -AssemblyName PresentationFramework
        [System.Windows.MessageBox]::Show(
            "Uninstall protection is on.`n`nOpen ContentGuard > Protection > Request uninstall. When the request is approved, run the uninstaller again.`n`n$out",
            "ContentGuard") | Out-Null
        Start-Process "http://127.0.0.1:17345/"
        exit 2
    }
}

Stop-Service -Name $service -Force -ErrorAction SilentlyContinue
sc.exe delete $service | Out-Null
if (Test-Path $exe) { & $exe cleanup }
Remove-Item "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ContentGuard" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path ([Environment]::GetFolderPath("CommonStartMenu")) "Programs\ContentGuard.url") -Force -ErrorAction SilentlyContinue
Remove-Item $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
# Settings in %ProgramData%\ContentGuard are kept so a reinstall keeps your lists.
Write-Host "ContentGuard removed."
