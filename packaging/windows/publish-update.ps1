# Publishes the installer in dist\ as an update: every user's app shows
# "Update available" (and updates itself) within a few hours, or right away
# when they press "Update now". Their settings are kept.
#
#   powershell -ExecutionPolicy Bypass -File packaging\windows\publish-update.ps1
#
# What it does, in order:
#   1. reads the version from dist\HTL-Content-Guard-PC-Setup.exe
#   2. asks "what's new" (English + Bangla) in a small window
#   3. signs the file with your offline key (HTL-Signing\update-signing-key.pk8)
#   4. uploads it to your Supabase storage (bucket "releases")
#   5. adds the version to the app_releases table
#   6. downloads it back and checks it
#
# The first time it asks for your Supabase SECRET key (Project Settings -> API
# Keys -> secret key). It is stored encrypted for your Windows account only.
param(
    [string]$Installer = (Join-Path $PSScriptRoot "..\..\dist\HTL-Content-Guard-PC-Setup.exe"),
    [string]$Key = (Join-Path $env:USERPROFILE "HTL-Signing\update-signing-key.pk8"),
    [int]$Rollout = 100,
    [string]$Notes = "",
    [string]$NotesBn = "",
    [switch]$DryRun
)
$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ProjectRef = "qhplhhrmathbfyejkzbr"
$Base = "https://$ProjectRef.supabase.co"
$SecretFile = Join-Path $env:USERPROFILE "HTL-Signing\supabase-secret.txt"

function Step($n, $text) { Write-Host ""; Write-Host "[$n/6] $text" -ForegroundColor Cyan }

# 1. version ---------------------------------------------------------------
Step 1 "Reading the installer"
$Installer = (Resolve-Path $Installer).Path
$ver = (Get-Item $Installer).VersionInfo.FileVersion
if ($ver -notmatch '^\d+(\.\d+){1,3}$') { throw "Could not read the version from $Installer" }
Write-Host "  Version $ver  ($([math]::Round((Get-Item $Installer).Length / 1MB, 1)) MB)"

# 2. what's new ------------------------------------------------------------
Step 2 "What's new in this version"
if (-not $Notes -or -not $NotesBn) {
    Add-Type -AssemblyName Microsoft.VisualBasic
    if (-not $Notes) { $Notes = [Microsoft.VisualBasic.Interaction]::InputBox("What's new in version $ver? (English, one short line)", "HTL Content Guard - publish update", "Improvements and fixes") }
    if (-not $NotesBn) { $NotesBn = [Microsoft.VisualBasic.Interaction]::InputBox("এই সংস্করণে কী নতুন? (বাংলায়, এক লাইনে)", "HTL Content Guard - publish update", "উন্নতি ও সমস্যার সমাধান") }
}
Write-Host "  EN: $Notes"
Write-Host "  BN: (saved)"

# 3. sign --------------------------------------------------------------------
Step 3 "Signing with your update key"
if (-not (Test-Path $Key)) { throw "Signing key not found: $Key  (restore it with packaging\signing\restore-key.ps1)" }
$agent = Join-Path $PSScriptRoot "..\..\target\release\contentguard-agent.exe"
if (-not (Test-Path $agent)) { $agent = "C:\Program Files\ContentGuard\contentguard-agent.exe" }
$signed = (& $agent update-sign $Installer $ver $Key | Out-String) | ConvertFrom-Json
if (-not $signed.signature) { throw "Signing failed" }
Write-Host "  sha256 $($signed.sha256)"

$fileUrl = "$Base/storage/v1/object/public/releases/v$ver/HTL-Content-Guard-PC-Setup.exe"
$row = [ordered]@{
    version = $ver; url = $fileUrl; sha256 = $signed.sha256; size = [int64]$signed.size
    signature = $signed.signature; rollout = $Rollout; enabled = $true; notes = $Notes; notes_bn = $NotesBn
}
if ($DryRun) {
    Write-Host ""
    Write-Host "DRY RUN - nothing uploaded. Would publish:" -ForegroundColor Yellow
    $row | ConvertTo-Json
    return
}

# Supabase secret key (stored encrypted with Windows DPAPI, this user only).
if (Test-Path $SecretFile) {
    $sec = Get-Content $SecretFile | ConvertTo-SecureString
} else {
    $sec = Read-Host "Paste your Supabase SECRET key (Project Settings -> API Keys)" -AsSecureString
    New-Item -ItemType Directory -Force (Split-Path $SecretFile) | Out-Null
    $sec | ConvertFrom-SecureString | Set-Content $SecretFile
}
$secret = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($sec))
$auth = @{ apikey = $secret }
if (-not $secret.StartsWith("sb_secret_")) { $auth["Authorization"] = "Bearer $secret" }  # legacy service_role key

# 4. upload ------------------------------------------------------------------
Step 4 "Uploading to Supabase storage"
$uploadUrl = "$Base/storage/v1/object/releases/v$ver/HTL-Content-Guard-PC-Setup.exe"
try {
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri $uploadUrl -InFile $Installer `
        -Headers ($auth + @{ "x-upsert" = "false" }) -ContentType "application/octet-stream" | Out-Null
    Write-Host "  uploaded"
} catch {
    $msg = "$($_.Exception.Message) $($_.ErrorDetails.Message)"
    if ($msg -match "exist|Duplicate|409") {
        Write-Host "  this version is already uploaded - checking it is the same file"
    } elseif ($msg -match "401|403|signature|Invalid") {
        Remove-Item $SecretFile -ErrorAction SilentlyContinue
        throw "Supabase refused the secret key. Run again and paste the SECRET key (not the publishable one)."
    } else { throw "Upload failed: $msg" }
}

# 6 (early). the file online must be exactly the one we signed ---------------
$tmp = Join-Path $env:TEMP "htl-check-$ver.exe"
Invoke-WebRequest -UseBasicParsing -Uri $fileUrl -OutFile $tmp
$online = (Get-FileHash $tmp -Algorithm SHA256).Hash.ToLower()
Remove-Item $tmp
if ($online -ne $signed.sha256) {
    throw "Version $ver is already online with a DIFFERENT file. Increase the version number, rebuild and publish again."
}

# 5. table row -------------------------------------------------------------
Step 5 "Adding version $ver to the app_releases table"
$body = [Text.Encoding]::UTF8.GetBytes(($row | ConvertTo-Json -Compress))
Invoke-RestMethod -Method Post -Uri "$Base/rest/v1/app_releases?on_conflict=version" `
    -Headers ($auth + @{ Prefer = "resolution=merge-duplicates,return=minimal" }) `
    -ContentType "application/json; charset=utf-8" -Body $body | Out-Null

Step 6 "Checked: the online file matches the signed one"
Write-Host ""
Write-Host "DONE. Version $ver is published to $Rollout% of users." -ForegroundColor Green
Write-Host "Users see 'Update available' when their app next checks (within ~6 hours),"
Write-Host "or right away when they open the Premium page or press 'Check for updates'."
Write-Host "To pause it: Supabase -> Table Editor -> app_releases -> set 'enabled' to false."
