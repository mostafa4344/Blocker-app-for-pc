# Prepares an automatic-update release of HTL Content Guard PC.
#
#   powershell -ExecutionPolicy Bypass -File packaging\windows\make-release.ps1 `
#       -Version 1.3.0 -Url https://github.com/<owner>/<repo>/releases/download/v1.3.0/HTL-Content-Guard-PC-Setup.exe
#
# 1. Signs dist\HTL-Content-Guard-PC-Setup.exe with the offline update key
#    (C:\Users\<you>\HTL-Signing\update-signing-key.pk8 by default).
# 2. Writes dist\release-<version>.sql: run it in the Supabase SQL Editor.
#    It starts the rollout at 1 % of devices; raise it in the app_releases
#    table (Table Editor) to 10, 50, 100 when nothing goes wrong.
#
# Upload the installer to -Url (e.g. a GitHub Release asset) BEFORE running the
# SQL. The app re-checks size, SHA-256 and the signature, so a wrong upload is
# simply refused — it can never install a file you didn't sign.
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Url,
    [string]$Installer = (Join-Path $PSScriptRoot "..\..\dist\HTL-Content-Guard-PC-Setup.exe"),
    [string]$Key = (Join-Path $env:USERPROFILE "HTL-Signing\update-signing-key.pk8"),
    [int]$Rollout = 1,
    [string]$Notes = ""
)
$ErrorActionPreference = "Stop"
if ($Version -notmatch '^\d+(\.\d+){1,3}$') { throw "Version must look like 1.3.0" }
if ($Url -notlike "https://*") { throw "Url must start with https://" }
if (-not (Test-Path $Installer)) { throw "Installer not found: $Installer" }
if (-not (Test-Path $Key)) { throw "Signing key not found: $Key" }

$agent = Join-Path $PSScriptRoot "..\..\target\release\contentguard-agent.exe"
if (-not (Test-Path $agent)) { throw "Build first: cargo build --release -p contentguard-agent" }

$json = & $agent update-sign $Installer $Version $Key
if ($LASTEXITCODE -ne 0) { throw "signing failed" }
$r = $json | ConvertFrom-Json

$q = { param($s) "'" + ($s -replace "'", "''") + "'" }
$sql = @"
-- Release $Version of HTL Content Guard PC (generated $(Get-Date -Format s)).
-- Rollout starts at $Rollout%. Raise app_releases.rollout later (10 -> 50 -> 100).
insert into public.app_releases (version, url, sha256, size, signature, rollout, notes)
values ($(& $q $r.version), $(& $q $Url), $(& $q $r.sha256), $($r.size), $(& $q $r.signature), $Rollout, $(& $q $Notes))
on conflict (version) do update
  set url = excluded.url, sha256 = excluded.sha256, size = excluded.size,
      signature = excluded.signature, notes = excluded.notes, enabled = true;
"@
$out = Join-Path (Split-Path $Installer) "release-$Version.sql"
Set-Content -Path $out -Value $sql -Encoding UTF8
Write-Host "Signed $Version  sha256=$($r.sha256)  size=$($r.size)"
Write-Host "1) Upload the installer to: $Url"
Write-Host "2) Run in Supabase SQL Editor: $out"
