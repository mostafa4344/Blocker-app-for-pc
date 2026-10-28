# Restores the update-signing key from the locked text made by backup-key.ps1
# (e.g. after reinstalling Windows or on a new PC).
param(
    [string]$InFile = (Join-Path ([Environment]::GetFolderPath("Desktop")) "HTL-signing-key-LOCKED.txt"),
    [string]$KeyFile = (Join-Path $env:USERPROFILE "HTL-Signing\update-signing-key.pk8"),
    [string]$Password = ""   # leave empty: you will be asked (recommended)
)
$ErrorActionPreference = "Stop"
if (Test-Path $KeyFile) { throw "A key already exists at $KeyFile - not overwriting it." }
if (-not (Test-Path $InFile)) { throw "Put the locked text in $InFile first (one line starting with HTLKEY1:)." }
$text = (Get-Content $InFile -Raw).Trim()
if (-not $text.StartsWith("HTLKEY1:")) { throw "This is not a locked HTL key (must start with HTLKEY1:)." }

if (-not $Password) {
    $p = Read-Host "Password" -AsSecureString
    $Password = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p))
}

$all = [Convert]::FromBase64String($text.Substring(8))
$body = $all[0..($all.Length - 33)]
$mac = $all[($all.Length - 32)..($all.Length - 1)]
$salt = $body[0..15]; $iv = $body[16..31]; $cipher = $body[32..($body.Length - 1)]

$kdf = New-Object Security.Cryptography.Rfc2898DeriveBytes($Password, [byte[]]$salt, 600000, [Security.Cryptography.HashAlgorithmName]::SHA256)
$encKey = $kdf.GetBytes(32); $macKey = $kdf.GetBytes(32)
$check = (New-Object Security.Cryptography.HMACSHA256(,$macKey)).ComputeHash([byte[]]$body)
if ([Convert]::ToBase64String($check) -ne [Convert]::ToBase64String([byte[]]$mac)) { throw "Wrong password (or the text was changed)." }

$aes = [Security.Cryptography.Aes]::Create()
$aes.Key = $encKey; $aes.IV = [byte[]]$iv
$aes.Mode = "CBC"; $aes.Padding = "PKCS7"
$plain = $aes.CreateDecryptor().TransformFinalBlock([byte[]]$cipher, 0, $cipher.Length)

New-Item -ItemType Directory -Force (Split-Path $KeyFile) | Out-Null
[IO.File]::WriteAllBytes($KeyFile, $plain)
Write-Host "DONE. Key restored to $KeyFile"
