//! A stable per-install identifier, used to anchor the free trial so that
//! wiping the app's data does not hand out a fresh trial.
//!
//! Two independent anchors make the id sticky:
//!  1. The OS machine id (salted + hashed so the raw id never leaves the device).
//!  2. A hidden file in the user's home directory, kept OUTSIDE the app data
//!     dir, so clearing the app's data (or, on Linux, regenerating
//!     /etc/machine-id) still keeps the same trial.
//!
//! This is a strong deterrent, not perfect DRM: a full OS reinstall that also
//! wipes the home directory changes the id. See NOTES.md.

use std::path::PathBuf;

use sha2::{Digest, Sha256};

const SALT: &str = "contentguard/v1/install";

/// Raw OS machine identifier, if one can be read without elevated rights.
fn machine_id() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        for p in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(s) = std::fs::read_to_string(p) {
                let s = s.trim();
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        // Line looks like:  "IOPlatformUUID" = "XXXXXXXX-XXXX-..."
        let line = text.lines().find(|l| l.contains("IOPlatformUUID"))?;
        let uuid = line.split('=').nth(1)?.trim().trim_matches('"').to_string();
        (!uuid.is_empty()).then_some(uuid)
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let out = std::process::Command::new("reg")
            .args([
                "query",
                r"HKLM\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        // Line looks like:  MachineGuid    REG_SZ    xxxxxxxx-xxxx-...
        let line = text.lines().find(|l| l.contains("MachineGuid"))?;
        let guid = line.split_whitespace().last()?.to_string();
        (guid.len() >= 16).then_some(guid)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn hashed(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(SALT.as_bytes());
    h.update(raw.as_bytes());
    let digest = h.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    // 32 hex chars is plenty and keeps within the backend's length limits.
    format!("cg1{}", &hex[..32])
}

/// A stable install id derived from the machine, or `None` if unavailable
/// (the caller then falls back to the anchor file or a random persisted id).
pub fn install_id() -> Option<String> {
    machine_id().map(|raw| hashed(&raw))
}

/// Path of the out-of-band id anchor. Overridable with `CONTENTGUARD_ID_FILE`
/// (used by tests and advanced setups). Defaults to a hidden file in the user's
/// home directory, deliberately separate from the app data dir.
fn anchor_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CONTENTGUARD_ID_FILE") {
        if !p.trim().is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)?;
    Some(home.join(".contentguard-id"))
}

/// Shape check so a corrupted or hostile anchor file can't inject arbitrary
/// data into the backend id.
fn looks_like_id(s: &str) -> bool {
    let s = s.trim();
    (8..=128).contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn read_anchor() -> Option<String> {
    let s = std::fs::read_to_string(anchor_path()?).ok()?;
    let s = s.trim().to_string();
    looks_like_id(&s).then_some(s)
}

fn write_anchor(id: &str) {
    if let Some(path) = anchor_path() {
        // Best effort: never fail startup just because the anchor can't be written.
        if read_anchor().as_deref() != Some(id) {
            let _ = std::fs::write(&path, id);
        }
    }
}

/// Resolve the install id to use and mirror it to the stable anchor.
///
/// Preference order: an id already saved in the license file, a previously
/// anchored id, the machine-derived id, then a freshly generated random id.
/// The anchor is checked before the machine id so that once an install is
/// known, the trial survives even if the OS machine id later changes.
pub fn resolve_install_id(existing: &str) -> String {
    let existing = existing.trim();
    let id = if !existing.is_empty() {
        existing.to_string()
    } else if let Some(a) = read_anchor() {
        a
    } else if let Some(m) = install_id() {
        m
    } else {
        format!(
            "{}{}",
            contentguard_core::new_id(),
            contentguard_core::new_id()
        )
    };
    write_anchor(&id);
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_shaped() {
        let a = hashed("some-machine-uuid");
        assert_eq!(a, hashed("some-machine-uuid"));
        assert_ne!(a, hashed("other-uuid"));
        assert!(a.starts_with("cg1"));
        assert_eq!(a.len(), 35);
    }

    #[test]
    fn reads_this_machine_on_linux() {
        // On the CI/dev Linux box a machine id should exist; elsewhere this is lenient.
        if cfg!(target_os = "linux") && std::path::Path::new("/etc/machine-id").exists() {
            assert!(install_id().is_some());
        }
    }

    #[test]
    fn anchor_persists_and_survives_app_data_wipe() {
        let dir = std::env::temp_dir().join(format!("cg-anchor-{}", contentguard_core::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(".contentguard-id");
        std::env::set_var("CONTENTGUARD_ID_FILE", &file);

        // First run: no existing id, no anchor yet -> an id is chosen and anchored.
        let first = resolve_install_id("");
        assert!(file.exists(), "anchor file should be written");
        assert!(looks_like_id(&first));

        // Simulate an app-data wipe: license file gone (existing empty). The same
        // id must come back from the anchor, not a fresh trial.
        let after_wipe = resolve_install_id("");
        assert_eq!(first, after_wipe);

        // An id already in the license file always wins and is re-anchored.
        let kept = resolve_install_id("cg1keepthisexactvalue0000000000000");
        assert_eq!(kept, "cg1keepthisexactvalue0000000000000");

        std::env::remove_var("CONTENTGUARD_ID_FILE");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_malformed_anchor() {
        assert!(!looks_like_id(""));
        assert!(!looks_like_id("short"));
        assert!(!looks_like_id("has spaces and $ymbols!"));
        assert!(looks_like_id("cg1abcdef0123456789abcdef01234567"));
    }
}
