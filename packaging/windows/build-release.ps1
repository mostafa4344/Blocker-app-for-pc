# Builds a release of HTL Content Guard PC on Windows, optionally code-signed.
#
#   # unsigned (no certificate yet):
#   powershell -ExecutionPolicy Bypass -File packaging\windows\build-release.ps1
#
#   # signed with your code-signing certificate (e.g. Certum via SimplySign):
#   powershell -ExecutionPolicy Bypass -File packaging\windows\build-release.ps1 -CertThumbprint <SHA1 thumbprint>
#
# Find the thumbprint after SimplySign Desktop is connected:
#   Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Format-List Subject, Thumbprint, NotAfter
#
# Output in dist\: HTL-Content-Guard-PC-Setup.exe, ContentGuard-windows.zip and
# content-guard-extension.zip. Afterwards publish the automatic update with
# make-release.ps1 (it signs the FINAL installer, so always run it last).
param(
    [string]$CertThumbprint = "",
    [string]$TimestampUrl = "http://time.certum.pl",
    [string]$SignTool = "",
    [string]$Makensis = (Join-Path $env:USERPROFILE "tools\nsis-3.11\makensis.exe")
)
$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\tools\winlibs\mingw64\bin;$env:PATH"

function Find-SignTool {
    if ($SignTool -and (Test-Path $SignTool)) { return $SignTool }
    $c = Get-ChildItem "$env:USERPROFILE\tools\sdk-buildtools\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1
    if ($c) { return $c.FullName }
    $c = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1
    if ($c) { return $c.FullName }
    throw "signtool.exe not found (see docs/CODE-SIGNING.md)"
}

$signCmd = $null
if ($CertThumbprint) {
    $st = Find-SignTool
    $cert = Get-ChildItem Cert:\CurrentUser\My, Cert:\LocalMachine\My -ErrorAction SilentlyContinue |
        Where-Object { $_.Thumbprint -eq $CertThumbprint.ToUpper() } | Select-Object -First 1
    if (-not $cert) { throw "Certificate $CertThumbprint not found. Is SimplySign Desktop connected?" }
    Write-Host "Signing with: $($cert.Subject)  (valid until $($cert.NotAfter))"
    $signCmd = "`"$st`" sign /sha1 $CertThumbprint /fd sha256 /tr $TimestampUrl /td sha256 /d `"HTL Content Guard PC`""
    # NSIS gets a tiny wrapper script: Windows PowerShell 5 mangles quoted
    # arguments passed to native programs, so the command can't go inline.
    $signScript = Join-Path $env:TEMP "htl-sign.cmd"
    Set-Content -Path $signScript -Value "@$signCmd %1" -Encoding ASCII
}
function Sign-File($path) {
    if (-not $signCmd) { return }
    cmd /c "$signCmd `"$path`""
    if ($LASTEXITCODE -ne 0) { throw "signing failed: $path" }
}

Write-Host "== checks"
cargo fmt --all --check; if ($LASTEXITCODE) { throw "cargo fmt" }
cargo clippy --all-targets -- -D warnings; if ($LASTEXITCODE) { throw "clippy" }
cargo test; if ($LASTEXITCODE) { throw "tests" }
node --test extension/test/*.test.js ui/test/*.test.mjs; if ($LASTEXITCODE) { throw "js tests" }

Write-Host "== build"
cargo build --release -p contentguard-agent; if ($LASTEXITCODE) { throw "build" }
$agent = Join-Path $root "target\release\contentguard-agent.exe"
Sign-File $agent

Write-Host "== installer"
$nsisArgs = @("-V2", "-DAGENT=$agent", "-DEXTDIR=$(Join-Path $root 'extension')")
if ($signCmd) { $nsisArgs += "-DSIGN_CMD=$signScript" }
Push-Location packaging\windows
& $Makensis @nsisArgs installer.nsi; if ($LASTEXITCODE) { Pop-Location; throw "makensis" }
Pop-Location
Copy-Item packaging\windows\HTL-Content-Guard-PC-Setup.exe dist\ -Force

Write-Host "== zips"
$build = Join-Path $root ".dev\build"
Remove-Item $build -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$build\ContentGuard", "$build\ext" | Out-Null
Copy-Item $agent, packaging\windows\install.ps1, packaging\windows\uninstall.ps1 "$build\ContentGuard\"
Copy-Item extension "$build\ContentGuard\extension" -Recurse
Remove-Item "$build\ContentGuard\extension\test" -Recurse -Force
Copy-Item extension\* "$build\ext\" -Recurse
Remove-Item "$build\ext\test" -Recurse -Force
python packaging\mkzip.py dist\ContentGuard-windows.zip "$build\ContentGuard" ContentGuard
python packaging\mkzip.py dist\content-guard-extension.zip "$build\ext"

$sig = Get-AuthenticodeSignature dist\HTL-Content-Guard-PC-Setup.exe
Write-Host "Done. Setup.exe signature: $($sig.Status) $($sig.SignerCertificate.Subject)"
