# Stops the demo app + license mock, wipes their data, starts them fresh.
$repo = "C:\Users\user\HTL-Content-Guard\Blocker-app-for-pc-ccr-00759c63-vxxesd"
Get-Process contentguard-agent -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "*target\debug*" } | Stop-Process -Force
Get-CimInstance Win32_Process -Filter "Name='node.exe'" | Where-Object { $_.CommandLine -like "*src/server.mjs*" } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Sleep -Milliseconds 800
Remove-Item -Recurse -Force "$repo\.dev\demo\data", "$repo\.dev\demo\backend", "$repo\.dev\demo\id" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "C:\Users\user\AppData\Local\Temp\cgvideo\profile" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$repo\.dev\demo\data", "$repo\.dev\demo\backend" | Out-Null
Set-Content "$repo\.dev\demo\hosts" "" -NoNewline
$env:PORT = "8788"; $env:DATA_DIR = "..\.dev\demo\backend"
Start-Process node -ArgumentList "src/server.mjs" -WorkingDirectory "$repo\backend" -WindowStyle Hidden -RedirectStandardOutput "C:\Users\user\AppData\Local\Temp\cgvideo\mock.log"
Start-Sleep -Seconds 1
$env:CONTENTGUARD_DATA_DIR = ".\.dev\demo\data"; $env:CONTENTGUARD_HOSTS_FILE = ".\.dev\demo\hosts"
$env:CONTENTGUARD_PORT = "17346"; $env:CONTENTGUARD_ID_FILE = ".\.dev\demo\id"; $env:CONTENTGUARD_SYNC_SECS = "10"
$env:CONTENTGUARD_BACKEND_URL = "http://127.0.0.1:8788"; $env:CONTENTGUARD_BACKEND_KEY = "dev"
$env:CONTENTGUARD_UI_DIR = ".\ui"; $env:CONTENTGUARD_UPDATE_DRY_RUN = "1"; $env:CONTENTGUARD_UPDATE_ALLOW_HTTP = "1"
$env:CONTENTGUARD_LICENSE_PUBKEY = "2152f8d19b791d24453242e15f2eab6cb7cffa7b6a5ed30097960e069881db12"
Start-Process "$repo\target\debug\contentguard-agent.exe" -ArgumentList "run" -WorkingDirectory $repo -WindowStyle Hidden -RedirectStandardError "C:\Users\user\AppData\Local\Temp\cgvideo\agent.log"
for ($i = 0; $i -lt 30; $i++) {
  try { Invoke-RestMethod -Uri "http://127.0.0.1:17346/api/status" -Headers @{ "X-ContentGuard" = "1" } -TimeoutSec 2 | Out-Null; Write-Output "demo ready"; break } catch { Start-Sleep -Milliseconds 500 }
}
