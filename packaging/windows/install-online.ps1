# Content Guard - one-line online installer.
# In normal PowerShell run:
#   irm 'https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/packaging/windows/install-online.ps1' | iex
$ErrorActionPreference = "Stop"
$zipUrl = "https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/dist/ContentGuard-windows.zip"
$tmp = Join-Path $env:TEMP "ContentGuard-dl"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zip = Join-Path $tmp "cg.zip"
Write-Host "Downloading Content Guard..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $zipUrl -OutFile $zip
$dst = Join-Path $tmp "pkg"
Remove-Item -Recurse -Force $dst -ErrorAction SilentlyContinue
Expand-Archive -Path $zip -DestinationPath $dst -Force
$installer = Join-Path $dst "ContentGuard\install.ps1"
if (!(Test-Path $installer)) { throw "installer not found in package" }
Write-Host "Launching installer (approve the Administrator prompt)..." -ForegroundColor Cyan
Start-Process powershell -Verb RunAs -ArgumentList @("-ExecutionPolicy","Bypass","-File","`"$installer`"")
Write-Host "Done. After it finishes, the app opens at http://127.0.0.1:17345/" -ForegroundColor Green
