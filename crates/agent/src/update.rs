//! Automatic updates of the desktop app.
//!
//! The license answer from the server carries the latest release
//! (`latest_release`: version, download URL, SHA-256, size, rollout % and an
//! Ed25519 signature). The agent:
//!  1. ignores it unless the version is newer and this device falls inside the
//!     staged-rollout percentage;
//!  2. downloads the installer into `<install dir>\updates` (Program Files —
//!     ordinary users can't write there, so the file can't be swapped);
//!  3. checks size, SHA-256 and the **Ed25519 signature against the public key
//!     compiled into the app** — the private key never leaves the publisher's
//!     PC, so even a compromised server or download host can't push code;
//!  4. runs the installer silently (`/S`); it upgrades in place and keeps
//!     settings.
//!
//! Each version is attempted at most a few times so a broken release can't
//! loop. `CONTENTGUARD_UPDATE_DRY_RUN=1` stops after verification (dev/tests).

use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Public half of the release-signing key (hex, 32 bytes Ed25519).
/// The private half is kept offline by the publisher (see docs/AUTO-UPDATE.md).
pub const UPDATE_PUBLIC_KEY_HEX: &str =
    "9290048a36c85cfc4bd1802447fd8b08296c0a2f456f4586cd6cb8ac16ce3bbc";

/// Domain-separation prefix of the signed message.
const SIGNED_PREFIX: &str = "htl-content-guard-update-v1";
/// Installers larger than this are refused (ours is ~4 MB).
const MAX_SIZE: u64 = 200 * 1024 * 1024;
/// Attempts per version before giving up on it.
pub const MAX_ATTEMPTS: u32 = 3;

/// A release as announced by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Release {
    pub version: String,
    pub url: String,
    /// Lower-case hex SHA-256 of the installer.
    pub sha256: String,
    pub size: u64,
    /// Hex Ed25519 signature of [`signed_message`].
    pub signature: String,
    /// 0–100: share of devices that get this release now.
    pub rollout: u8,
    /// "What's new", shown in the app (English / Bangla).
    pub notes: String,
    pub notes_bn: String,
}

/// The exact bytes the publisher signs.
pub fn signed_message(version: &str, sha256: &str, size: u64) -> Vec<u8> {
    format!(
        "{SIGNED_PREFIX}\n{version}\n{}\n{size}",
        sha256.to_ascii_lowercase()
    )
    .into_bytes()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn unhex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// `1.2.10` > `1.2.9`. Non-numeric parts count as 0; extra parts compare too.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim()
            .trim_start_matches('v')
            .split(['.', '-', '+'])
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parse(candidate), parse(current));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (
            a.get(i).copied().unwrap_or(0),
            b.get(i).copied().unwrap_or(0),
        );
        if x != y {
            return x > y;
        }
    }
    false
}

/// Stable 0–99 bucket for this device and release, so a 10 % rollout reaches
/// the same 10 % of devices on every check (and a different 10 % next release).
pub fn rollout_bucket(install_id: &str, version: &str) -> u8 {
    let d = Sha256::digest(format!("{install_id}/{version}").as_bytes());
    (u16::from_be_bytes([d[0], d[1]]) % 100) as u8
}

pub fn in_rollout(install_id: &str, r: &Release) -> bool {
    rollout_bucket(install_id, &r.version) < r.rollout.min(100)
}

/// Verifies the publisher's signature over (version, sha256, size).
pub fn signature_ok(r: &Release, public_key_hex: &str) -> bool {
    use ring::signature::{UnparsedPublicKey, ED25519};
    let (Some(key), Some(sig)) = (unhex(public_key_hex), unhex(&r.signature)) else {
        return false;
    };
    if key.len() != 32 || key.iter().all(|b| *b == 0) {
        return false; // no real key configured: never update
    }
    UnparsedPublicKey::new(&ED25519, key)
        .verify(&signed_message(&r.version, &r.sha256, r.size), &sig)
        .is_ok()
}

/// Whether the announced release is a genuine, newer version this device
/// could install (shown to the user as "update available").
pub fn available(r: &Release, current: &str) -> bool {
    !r.version.is_empty()
        && is_newer(&r.version, current)
        && (r.url.starts_with("https://") || dev_http_allowed(&r.url))
        && r.size > 0
        && r.size <= MAX_SIZE
        && signature_ok(r, UPDATE_PUBLIC_KEY_HEX)
}

/// Whether the background updater should install it now (inside the rollout).
pub fn wanted(r: &Release, current: &str, install_id: &str) -> bool {
    available(r, current) && in_rollout(install_id, r)
}

/// Development only: allow a plain-HTTP download from this machine when
/// `CONTENTGUARD_UPDATE_ALLOW_HTTP` is set (never set for the real service).
/// The signature check still applies.
fn dev_http_allowed(url: &str) -> bool {
    std::env::var_os("CONTENTGUARD_UPDATE_ALLOW_HTTP").is_some()
        && url.starts_with("http://127.0.0.1:")
}

fn updates_dir() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join("updates");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Downloads, verifies and starts the installer. Returns a short status.
pub fn download_and_install(r: &Release) -> Result<&'static str, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(600)))
        .http_status_as_error(true)
        .build()
        .into();
    let mut resp = agent.get(&r.url).call().map_err(|e| e.to_string())?;
    let mut bytes = Vec::with_capacity(r.size as usize);
    resp.body_mut()
        .as_reader()
        .take(r.size + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 != r.size {
        return Err(format!("size mismatch: got {} bytes", bytes.len()));
    }
    let digest = hex(&Sha256::digest(&bytes));
    if !digest.eq_ignore_ascii_case(r.sha256.trim()) {
        return Err("sha256 mismatch".into());
    }
    // Re-check the signature against what we will actually run.
    if !signature_ok(r, UPDATE_PUBLIC_KEY_HEX) {
        return Err("bad signature".into());
    }
    let dir = updates_dir().ok_or("no updates dir")?;
    let safe_version: String = r
        .version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.')
        .collect();
    let path = dir.join(format!("HTL-Content-Guard-PC-Setup-{safe_version}.exe"));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    if std::env::var("CONTENTGUARD_UPDATE_DRY_RUN").is_ok() {
        return Ok("verified (dry run)");
    }
    start_installer(&path)?;
    Ok("installer started")
}

#[cfg(windows)]
fn start_installer(path: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    // Detached so it survives the installer stopping this agent.
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new(path)
        .arg("/S")
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn start_installer(_path: &std::path::Path) -> Result<(), String> {
    Err("automatic install is only implemented on Windows".into())
}

// ---------- publisher tools (CLI) ----------

/// Creates a new signing key: writes the private key (PKCS#8) to `out` and
/// returns the public key hex to paste into `UPDATE_PUBLIC_KEY_HEX`.
pub fn keygen(out: &std::path::Path) -> Result<String, String> {
    use ring::signature::{Ed25519KeyPair, KeyPair};
    if out.exists() {
        return Err(format!(
            "{} already exists — refusing to overwrite a signing key",
            out.display()
        ));
    }
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| e.to_string())?;
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).map_err(|e| e.to_string())?;
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(out, pkcs8.as_ref()).map_err(|e| e.to_string())?;
    Ok(hex(pair.public_key().as_ref()))
}

/// Signs an installer: returns the release fields (without URL / rollout).
pub fn sign(
    installer: &std::path::Path,
    version: &str,
    key: &std::path::Path,
) -> Result<Release, String> {
    use ring::signature::Ed25519KeyPair;
    let bytes = std::fs::read(installer).map_err(|e| e.to_string())?;
    let pkcs8 = std::fs::read(key).map_err(|e| e.to_string())?;
    let pair = Ed25519KeyPair::from_pkcs8(&pkcs8).map_err(|e| e.to_string())?;
    let sha = hex(&Sha256::digest(&bytes));
    let size = bytes.len() as u64;
    let sig = pair.sign(&signed_message(version, &sha, size));
    Ok(Release {
        version: version.to_string(),
        sha256: sha,
        size,
        signature: hex(sig.as_ref()),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer("1.2.10", "1.2.9"));
        assert!(is_newer("1.3.0", "1.2.99"));
        assert!(is_newer("2.0", "1.9.9"));
        assert!(!is_newer("1.2.0", "1.2.0"));
        assert!(!is_newer("1.1.9", "1.2.0"));
        assert!(is_newer("v1.2.1", "1.2.0"));
    }

    #[test]
    fn rollout_is_stable_and_proportional() {
        let r = Release {
            version: "9.9.9".into(),
            rollout: 10,
            ..Default::default()
        };
        let ids: Vec<String> = (0..2000).map(|i| format!("device-{i}")).collect();
        let hit = ids.iter().filter(|id| in_rollout(id, &r)).count();
        assert!((120..=280).contains(&hit), "about 10% expected, got {hit}");
        assert_eq!(in_rollout("device-7", &r), in_rollout("device-7", &r));
        let all = Release {
            rollout: 100,
            ..r.clone()
        };
        assert!(ids.iter().all(|id| in_rollout(id, &all)));
        let none = Release { rollout: 0, ..r };
        assert!(!ids.iter().any(|id| in_rollout(id, &none)));
    }

    #[test]
    fn signature_round_trip_and_tampering() {
        let dir = std::env::temp_dir().join(format!("cg-upd-{}", contentguard_core::new_id()));
        let key = dir.join("k.pk8");
        let pubkey = keygen(&key).unwrap();
        assert!(keygen(&key).is_err(), "never overwrites a key");
        let exe = dir.join("setup.exe");
        std::fs::write(&exe, b"pretend installer").unwrap();
        let mut r = sign(&exe, "1.9.0", &key).unwrap();
        r.url = "https://example.com/setup.exe".into();
        r.rollout = 100;
        assert!(signature_ok(&r, &pubkey));
        // Any change to version, hash or size breaks the signature.
        assert!(!signature_ok(
            &Release {
                version: "1.9.1".into(),
                ..r.clone()
            },
            &pubkey
        ));
        assert!(!signature_ok(
            &Release {
                size: r.size + 1,
                ..r.clone()
            },
            &pubkey
        ));
        let mut other = r.sha256.clone();
        other.replace_range(0..1, if other.starts_with('0') { "1" } else { "0" });
        assert!(!signature_ok(
            &Release {
                sha256: other,
                ..r.clone()
            },
            &pubkey
        ));
        // The all-zero placeholder key never accepts anything.
        assert!(!signature_ok(&r, &"00".repeat(32)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hex_round_trip() {
        assert_eq!(
            unhex(&hex(&[0, 1, 254, 255])).unwrap(),
            vec![0, 1, 254, 255]
        );
        assert!(unhex("abc").is_none());
        assert!(unhex("zz").is_none());
    }
}
