//! A secret that exists only on this computer. The license server binds each
//! Device ID to a fingerprint of it (`device_key`), and signed licenses carry
//! that fingerprint, so copying license.json (or the Device ID) to another PC
//! doesn't bring Premium along.
//!
//! On Windows the secret is sealed with DPAPI in machine scope (it can't be
//! opened on another computer) and the file is readable only by SYSTEM and
//! Administrators.

use std::path::Path;

use sha2::{Digest, Sha256};

const FILE: &str = "device.key";
const DOMAIN: &str = "htl-content-guard-device-v1";

/// The public fingerprint (64 hex chars) of this computer's device secret,
/// creating the secret on first use. `None` only if it can't be stored.
pub fn load_or_create(dir: &Path) -> Option<String> {
    let path = dir.join(FILE);
    if let Ok(blob) = std::fs::read(&path) {
        match unseal(&blob) {
            Some(secret) if secret.len() == 32 => {
                restrict(&path);
                return Some(fingerprint(&secret));
            }
            // Sealed on another computer (copied) or damaged: keep it aside
            // and start a new one; the server decides whether it matches.
            _ => {
                let _ = std::fs::rename(&path, dir.join("device.key.invalid"));
            }
        }
    }
    let secret = new_secret();
    let blob = seal(&secret)?;
    // create_new: never write into a file someone else prepared.
    let _ = std::fs::remove_file(&path);
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        f.write_all(&blob).ok()?;
        f.sync_all().ok()?;
    }
    restrict(&path);
    // Read it back: only a secret that can be opened again counts.
    let check = unseal(&std::fs::read(&path).ok()?)?;
    (check == secret).then(|| fingerprint(&secret))
}

fn fingerprint(secret: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(DOMAIN.as_bytes());
    h.update(secret);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// 32 random bytes from the OS-seeded CSPRNG (via `new_id`, 8 bytes each).
fn new_secret() -> Vec<u8> {
    let hexed: String = (0..4).map(|_| contentguard_core::new_id()).collect();
    (0..32)
        .filter_map(|i| u8::from_str_radix(&hexed[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

/// Whether ACLs should be changed: not in development runs, where the agent
/// runs as the developer and data lives in a private folder.
fn managed_location(path: &Path) -> bool {
    std::env::var_os("CONTENTGUARD_DATA_DIR").is_none()
        && path.starts_with(crate::paths::data_dir())
}

/// Locks the data folder: SYSTEM and Administrators may change it, other
/// accounts may only read. (Windows' default for ProgramData lets every user
/// CREATE files there, which would let a child's account plant a file the
/// agent later writes its settings into.) Also takes back ownership of
/// anything already inside.
#[cfg(windows)]
pub fn protect_data_dir(dir: &Path) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    if !managed_location(dir) {
        return;
    }
    let icacls = |args: &[&std::ffi::OsStr]| {
        let _ = std::process::Command::new("icacls")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    };
    let d = dir.as_os_str();
    let all = dir.join("*");
    icacls(&[
        d,
        "/inheritance:r".as_ref(),
        "/grant:r".as_ref(),
        "*S-1-5-18:(OI)(CI)F".as_ref(),
        "*S-1-5-32-544:(OI)(CI)F".as_ref(),
        "*S-1-5-32-545:(OI)(CI)RX".as_ref(),
        "/Q".as_ref(),
    ]);
    icacls(&[
        all.as_os_str(),
        "/reset".as_ref(),
        "/T".as_ref(),
        "/C".as_ref(),
        "/Q".as_ref(),
    ]);
    icacls(&[
        d,
        "/setowner".as_ref(),
        "*S-1-5-18".as_ref(),
        "/T".as_ref(),
        "/C".as_ref(),
        "/Q".as_ref(),
    ]);
}

#[cfg(not(windows))]
pub fn protect_data_dir(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if managed_location(dir) {
        let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o755));
    }
}

#[cfg(windows)]
fn restrict(path: &Path) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    if !managed_location(path) {
        return;
    }
    // SYSTEM and Administrators only (no inherited "Users: read").
    let _ = std::process::Command::new("icacls")
        .arg(path)
        .args([
            "/inheritance:r",
            "/grant:r",
            "*S-1-5-18:F",
            "*S-1-5-32-544:F",
            "/Q",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}

#[cfg(not(windows))]
fn restrict(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if managed_location(path) {
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
}

#[cfg(windows)]
mod dpapi {
    use std::ffi::c_void;

    #[repr(C)]
    struct Blob {
        len: u32,
        data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            input: *const Blob,
            description: *const u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const Blob,
            description: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(mem: *mut c_void) -> *mut c_void;
    }

    const UI_FORBIDDEN: u32 = 0x1;
    const LOCAL_MACHINE: u32 = 0x4;
    const ENTROPY: &[u8] = b"htl-content-guard-device-v1";

    fn run(input: &[u8], seal: bool) -> Option<Vec<u8>> {
        let inp = Blob {
            len: input.len() as u32,
            data: input.as_ptr() as *mut u8,
        };
        let ent = Blob {
            len: ENTROPY.len() as u32,
            data: ENTROPY.as_ptr() as *mut u8,
        };
        let mut out = Blob {
            len: 0,
            data: std::ptr::null_mut(),
        };
        // SAFETY: all pointers are valid for the call; the output buffer is
        // allocated by the API and released with LocalFree below.
        let ok = unsafe {
            if seal {
                CryptProtectData(
                    &inp,
                    std::ptr::null(),
                    &ent,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    UI_FORBIDDEN | LOCAL_MACHINE,
                    &mut out,
                )
            } else {
                CryptUnprotectData(
                    &inp,
                    std::ptr::null_mut(),
                    &ent,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    UI_FORBIDDEN,
                    &mut out,
                )
            }
        };
        if ok == 0 || out.data.is_null() {
            return None;
        }
        // SAFETY: the API returned `out.len` bytes at `out.data`.
        let bytes = unsafe { std::slice::from_raw_parts(out.data, out.len as usize).to_vec() };
        unsafe { LocalFree(out.data as *mut c_void) };
        Some(bytes)
    }

    pub fn seal(data: &[u8]) -> Option<Vec<u8>> {
        run(data, true)
    }
    pub fn unseal(data: &[u8]) -> Option<Vec<u8>> {
        run(data, false)
    }
}

#[cfg(windows)]
fn seal(secret: &[u8]) -> Option<Vec<u8>> {
    dpapi::seal(secret)
}
#[cfg(windows)]
fn unseal(blob: &[u8]) -> Option<Vec<u8>> {
    dpapi::unseal(blob)
}

// Elsewhere the file permissions (root only, 0600) protect the secret.
#[cfg(not(windows))]
fn seal(secret: &[u8]) -> Option<Vec<u8>> {
    Some(secret.to_vec())
}
#[cfg(not(windows))]
fn unseal(blob: &[u8]) -> Option<Vec<u8>> {
    Some(blob.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_secret_same_fingerprint_and_survives_restart() {
        let dir = std::env::temp_dir().join(format!("cg-devkey-{}", contentguard_core::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = load_or_create(&dir).expect("created");
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        // Restart: the same secret is read back.
        assert_eq!(load_or_create(&dir).unwrap(), a);
        // A file that can't be opened (e.g. copied from another PC) is
        // replaced by a new secret: a different fingerprint.
        std::fs::write(dir.join(FILE), b"not a sealed secret").unwrap();
        let b = load_or_create(&dir).unwrap();
        if cfg!(windows) {
            assert_ne!(a, b);
            assert!(dir.join("device.key.invalid").exists());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn secrets_are_random() {
        let a = new_secret();
        assert_eq!(a.len(), 32);
        assert_ne!(a, new_secret());
        assert_ne!(fingerprint(&a), fingerprint(&new_secret()));
    }
}
