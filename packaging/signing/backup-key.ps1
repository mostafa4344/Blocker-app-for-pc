# Locks (encrypts) the update-signing key with YOUR password so the result can
# be kept safely in Google Drive / Google Keep / email. Without the password
# the text is useless to anyone. Restore with restore-key.ps1.
#
# Encryption: PBKDF2-SHA256 (600,000 rounds) -> AES-256-CBC + HMAC-SHA256.
param(
    [string]$KeyFile = (Join-Path $env:USERPROFILE "HTL-Signing\update-signing-key.pk8"),
    [string]$OutFile = (Join-Path ([Environment]::GetFolderPath("Desktop")) "HTL-signing-key-LOCKED.txt"),
    [string]$Password = ""   # leave empty: you will be asked (recommended)
)
$ErrorActionPreference = "Stop"
if (-not (Test-Path $KeyFile)) { throw "Key not found: $KeyFile" }

if (-not $Password) {
    $p1 = Read-Host "Choose a strong password (at least 12 characters)" -AsSecureString
    $p2 = Read-Host "Type the same password again" -AsSecureString
    $b1 = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p1))
    $b2 = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p2))
    if ($b1 -ne $b2) { throw "The two passwords are different. Run again." }
    $Password = $b1
}
if ($Password.Length -lt 12) { throw "Password too short: use at least 12 characters." }

$plain = [IO.File]::ReadAllBytes($KeyFile)
$rng = [Security.Cryptography.RandomNumberGenerator]::Create()
$salt = New-Object byte[] 16; $rng.GetBytes($salt)
$iv = New-Object byte[] 16; $rng.GetBytes($iv)
$kdf = New-Object Security.Cryptography.Rfc2898DeriveBytes($Password, $salt, 600000, [Security.Cryptography.HashAlgorithmName]::SHA256)
$encKey = $kdf.GetBytes(32); $macKey = $kdf.GetBytes(32)

$aes = [Security.Cryptography.Aes]::Create()
$aes.Key = $encKey; $aes.IV = $iv
$aes.Mode = "CBC"; $aes.Padding = "PKCS7"
$cipher = $aes.CreateEncryptor().TransformFinalBlock($plain, 0, $plain.Length)

$body = $salt + $iv + $cipher
$mac = (New-Object Security.Cryptography.HMACSHA256(,$macKey)).ComputeHash($body)
$text = "HTLKEY1:" + [Convert]::ToBase64String($body + $mac)

Set-Content -Path $OutFile -Value $text -Encoding ASCII
try { Set-Clipboard -Value $text } catch {}
Write-Host ""
Write-Host "DONE. The locked key was saved to:  $OutFile"
Write-Host "It is also copied - you can paste it (Ctrl+V) into Google Keep."
Write-Host "Keep your password somewhere else (NOT in the same place)."
