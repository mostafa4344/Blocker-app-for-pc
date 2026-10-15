//! A stable per-machine identifier, used to anchor the free trial so that
//! wiping the app's data does not hand out a fresh trial. Derived from the
//! OS machine id, then salted and hashed so the raw id never leaves the device.
//!
//! This is a strong deterrent, not perfect DRM: a full OS reinstall (or, on
//! Linux, regenerating /etc/machine-id) changes the id. See NOTES.md.

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
/// (the caller then falls back to a random persisted id).
pub fn install_id() -> Option<String> {
    machine_id().map(|raw| hashed(&raw))
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
}
