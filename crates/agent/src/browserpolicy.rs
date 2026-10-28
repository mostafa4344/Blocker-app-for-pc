//! Force-install the companion extension into supported browsers through the
//! OS's *official* enterprise browser-management policy — the same mechanism
//! schools and companies use. This is deliberately NOT a hack:
//!
//! * A force-installed extension installs itself on browser start (so the user
//!   only clicks one button in our app), and its "Remove" control is locked —
//!   the user cannot delete or disable it from `chrome://extensions`.
//! * The policy lives in the machine registry / managed-policy dir, independent
//!   of the browser profile, so it survives the browser being reinstalled.
//! * The agent re-asserts it every few seconds, so deleting the policy key (which
//!   needs admin anyway) is undone almost immediately.
//!
//! Prerequisite: a stable extension identity (`id` + update URL). Use a store
//! listing (Chrome Web Store / Edge Add-ons / AMO) or a self-hosted signed
//! build — see `packaging/extension/README.md`. Until an id is configured the
//! feature is inert (nothing is written).

// Which builders are "live" depends on the target OS (the Windows apply path
// uses the registry; the Linux path uses the managed JSON / policies.json), so
// some items are unused on any single platform. The module is small and purely
// declarative, so allow dead code rather than scatter per-item cfgs.
#![allow(dead_code)]

/// Chrome Web Store update service (used when `id` is a Web Store id).
pub const CHROME_WEBSTORE_UPDATE: &str = "https://clients2.google.com/service/update2/crx";
/// Microsoft Edge Add-ons update service.
pub const EDGE_WEBSTORE_UPDATE: &str = "https://edge.microsoft.com/extensionwebstorebase/v1/crx";

/// Firefox gecko id of the companion extension (from the extension manifest).
pub const FIREFOX_EXTENSION_ID: &str = "extension@contentguard.app";

/// Built-in Chrome Web Store id of the Content Guard extension. Used when the
/// user has not set a custom `extension_id`, so turning the switch on just works
/// on a normal install. (Assigned by the Web Store at upload; stable across
/// publishes. Covers Chrome, Edge, Brave, Vivaldi and Opera.)
pub const DEFAULT_CHROMIUM_ID: &str = "amlkhggimheblaboilcbjdjomlceckdp";

/// The three identifiers needed to force-install the extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Chromium extension id (32 chars, a–p) — a store id or a self-hosted id.
    pub chromium_id: String,
    /// Chromium update manifest URL (store service or a self-hosted update.xml).
    pub chromium_update_url: String,
    /// Firefox signed-XPI install URL (AMO or self-hosted). Empty disables Firefox.
    pub firefox_install_url: String,
}

impl Identity {
    /// Build an identity from the user's settings, falling back to a Web Store
    /// update URL when only an id is given. Returns `None` when no id is set
    /// (the feature stays inert).
    pub fn from_settings(id: &str, update_url: &str, firefox_xpi: &str) -> Option<Identity> {
        // Fall back to the built-in store id when the user hasn't set one.
        let id = {
            let t = id.trim();
            if t.is_empty() {
                DEFAULT_CHROMIUM_ID
            } else {
                t
            }
        };
        if !valid_chromium_id(id) {
            return None;
        }
        let url = update_url.trim();
        Some(Identity {
            chromium_id: id.to_string(),
            chromium_update_url: if url.is_empty() {
                CHROME_WEBSTORE_UPDATE.to_string()
            } else {
                url.to_string()
            },
            firefox_install_url: firefox_xpi.trim().to_string(),
        })
    }

    /// `"<id>;<update_url>"` — the value an `ExtensionInstallForcelist` entry holds.
    pub fn forcelist_entry(&self) -> String {
        format!("{};{}", self.chromium_id, self.chromium_update_url)
    }
}

/// A Chromium extension id is exactly 32 characters from the alphabet a–p.
pub fn valid_chromium_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| (b'a'..=b'p').contains(&b))
}

/// Chromium browsers that read enterprise policy, with their Windows registry
/// vendor\product path and their Linux managed-policy directory.
pub const CHROMIUM_TARGETS: &[(&str, &str, &str)] = &[
    (
        "Google Chrome",
        r"Google\Chrome",
        "/etc/opt/chrome/policies/managed",
    ),
    (
        "Microsoft Edge",
        r"Microsoft\Edge",
        "/etc/opt/edge/policies/managed",
    ),
    (
        "Brave",
        r"BraveSoftware\Brave",
        "/etc/brave/policies/managed",
    ),
    ("Chromium", r"Chromium", "/etc/chromium/policies/managed"),
    ("Vivaldi", r"Vivaldi", "/etc/opt/vivaldi/policies/managed"),
];

/// The managed-policy JSON a Chromium browser reads on Linux/macOS to force the
/// extension (and lock its removal).
pub fn chromium_managed_json(id: &Identity) -> String {
    format!(
        "{{\n  \"ExtensionInstallForcelist\": [\"{}\"]\n}}\n",
        id.forcelist_entry()
    )
}

/// The `ExtensionSettings` object (Firefox GPO value / policies.json fragment)
/// that force-installs and locks the extension. `None` without a Firefox URL.
pub fn firefox_extension_settings(id: &Identity) -> Option<String> {
    if id.firefox_install_url.is_empty() {
        return None;
    }
    Some(format!(
        "{{\"{fid}\":{{\"installation_mode\":\"force_installed\",\"install_url\":\"{url}\"}}}}",
        fid = FIREFOX_EXTENSION_ID,
        url = id.firefox_install_url,
    ))
}

/// The full `policies.json` Firefox reads on Linux/macOS. `None` without a URL.
pub fn firefox_policies_json(id: &Identity) -> Option<String> {
    let settings = firefox_extension_settings(id)?;
    Some(format!(
        "{{\n  \"policies\": {{\n    \"ExtensionSettings\": {settings}\n  }}\n}}\n"
    ))
}

// ---------------------------------------------------------------------------
// Applying / removing the policy (best-effort; needs admin rights).
// ---------------------------------------------------------------------------

/// Apply the force-install policy to every detected browser family. Safe to call
/// every tick: it only writes when the policy is missing or different.
pub fn apply(id: &Identity) {
    #[cfg(target_os = "windows")]
    windows::apply(id);
    #[cfg(target_os = "linux")]
    linux::apply(id);
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    let _ = id;
}

/// Remove the force-install policy (used on a legitimate uninstall / disable).
pub fn remove() {
    #[cfg(target_os = "windows")]
    windows::remove();
    #[cfg(target_os = "linux")]
    linux::remove();
}

/// Policy values that turn off private windows, per browser:
/// (Windows registry product path, value name, value meaning "disabled").
pub const PRIVATE_WINDOW_POLICIES: &[(&str, &str, u32)] = &[
    (r"Google\Chrome", "IncognitoModeAvailability", 1),
    (r"Microsoft\Edge", "InPrivateModeAvailability", 1),
    (r"BraveSoftware\Brave", "IncognitoModeAvailability", 1),
    (r"Chromium", "IncognitoModeAvailability", 1),
    (r"Vivaldi", "IncognitoModeAvailability", 1),
    (r"Mozilla\Firefox", "DisablePrivateBrowsing", 1),
];

/// Turn private / incognito windows off (`true`) or restore the default.
/// Safe to call repeatedly; only writes when a value is missing or different.
pub fn set_private_windows_blocked(blocked: bool) {
    #[cfg(target_os = "windows")]
    windows::set_private(blocked);
    #[cfg(not(target_os = "windows"))]
    let _ = blocked;
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{Identity, CHROMIUM_TARGETS};
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn reg(args: &[&str]) {
        let _ = Command::new("reg")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }

    fn read(path: &str, value: &str) -> Option<String> {
        let out = Command::new("reg")
            .args(["query", path, "/v", value])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text.lines().find(|l| l.contains(value))?;
        // `    1    REG_SZ    <id>;<url>`
        line.split("REG_SZ").nth(1).map(|s| s.trim().to_string())
    }

    pub fn apply(id: &Identity) {
        let entry = id.forcelist_entry();
        for (_, product, _) in CHROMIUM_TARGETS {
            let key = format!(r"HKLM\SOFTWARE\Policies\{product}\ExtensionInstallForcelist");
            // Only (re)write when our entry is not already present under "1".
            if read(&key, "1").as_deref() != Some(entry.as_str()) {
                reg(&["add", &key, "/v", "1", "/t", "REG_SZ", "/d", &entry, "/f"]);
            }
        }
        // Firefox reads ExtensionSettings as one REG_SZ of JSON under its key.
        if let Some(obj) = super::firefox_extension_settings(id) {
            let key = r"HKLM\SOFTWARE\Policies\Mozilla\Firefox";
            reg(&[
                "add",
                key,
                "/v",
                "ExtensionSettings",
                "/t",
                "REG_SZ",
                "/d",
                &obj,
                "/f",
            ]);
        }
    }

    fn read_dword(path: &str, value: &str) -> Option<u32> {
        let out = Command::new("reg")
            .args(["query", path, "/v", value])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text.lines().find(|l| l.contains(value))?;
        let hex = line
            .split("REG_DWORD")
            .nth(1)?
            .trim()
            .trim_start_matches("0x");
        u32::from_str_radix(hex, 16).ok()
    }

    pub fn set_private(blocked: bool) {
        for (product, value, disabled) in super::PRIVATE_WINDOW_POLICIES {
            let key = format!(r"HKLM\SOFTWARE\Policies\{product}");
            let current = read_dword(&key, value);
            if blocked && current != Some(*disabled) {
                let d = disabled.to_string();
                reg(&["add", &key, "/v", value, "/t", "REG_DWORD", "/d", &d, "/f"]);
            } else if !blocked && current.is_some() {
                reg(&["delete", &key, "/v", value, "/f"]);
            }
        }
    }

    pub fn remove() {
        for (_, product, _) in CHROMIUM_TARGETS {
            let key = format!(r"HKLM\SOFTWARE\Policies\{product}\ExtensionInstallForcelist");
            reg(&["delete", &key, "/f"]);
        }
        reg(&[
            "delete",
            r"HKLM\SOFTWARE\Policies\Mozilla\Firefox",
            "/v",
            "ExtensionSettings",
            "/f",
        ]);
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{Identity, CHROMIUM_TARGETS};
    use std::fs;
    use std::path::Path;

    const FILE: &str = "contentguard.json";
    const FIREFOX_DIRS: &[&str] = &[
        "/etc/firefox/policies",
        "/usr/lib/firefox/distribution",
        "/usr/lib/firefox-esr/distribution",
    ];

    pub fn apply(id: &Identity) {
        let json = super::chromium_managed_json(id);
        for (_, _, dir) in CHROMIUM_TARGETS {
            let path = Path::new(dir).join(FILE);
            if fs::read_to_string(&path).ok().as_deref() != Some(json.as_str())
                && fs::create_dir_all(dir).is_ok()
            {
                let _ = fs::write(&path, &json);
            }
        }
        if let Some(policies) = super::firefox_policies_json(id) {
            for dir in FIREFOX_DIRS {
                if fs::create_dir_all(dir).is_ok() {
                    let path = Path::new(dir).join("policies.json");
                    if fs::read_to_string(&path).ok().as_deref() != Some(policies.as_str()) {
                        let _ = fs::write(&path, &policies);
                    }
                }
            }
        }
    }

    pub fn remove() {
        for (_, _, dir) in CHROMIUM_TARGETS {
            let _ = fs::remove_file(Path::new(dir).join(FILE));
        }
        for dir in FIREFOX_DIRS {
            let _ = fs::remove_file(Path::new(dir).join("policies.json"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident() -> Identity {
        Identity {
            chromium_id: "abcdefghijklmnopabcdefghijklmnop".into(),
            chromium_update_url: "https://example.com/update.xml".into(),
            firefox_install_url: "https://example.com/cg.xpi".into(),
        }
    }

    #[test]
    fn id_validation() {
        assert!(valid_chromium_id("abcdefghijklmnopabcdefghijklmnop"));
        assert!(!valid_chromium_id("tooshort"));
        assert!(!valid_chromium_id("abcdefghijklmnopabcdefghijklmnoq")); // q is out of a–p
        assert!(!valid_chromium_id("ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP")); // upper-case
    }

    #[test]
    fn from_settings_needs_valid_id() {
        // Empty id falls back to the built-in store id.
        let def = Identity::from_settings("", "", "").unwrap();
        assert_eq!(def.chromium_id, DEFAULT_CHROMIUM_ID);
        assert!(valid_chromium_id(DEFAULT_CHROMIUM_ID));
        // A malformed custom id is rejected (no silent fallback that hides typos).
        assert!(Identity::from_settings("not-a-real-id", "", "").is_none());
        let i = Identity::from_settings("abcdefghijklmnopabcdefghijklmnop", "", "").unwrap();
        // Empty update url falls back to the Web Store service.
        assert_eq!(i.chromium_update_url, CHROME_WEBSTORE_UPDATE);
        assert!(i.firefox_install_url.is_empty());
    }

    #[test]
    fn forcelist_entry_shape() {
        assert_eq!(
            ident().forcelist_entry(),
            "abcdefghijklmnopabcdefghijklmnop;https://example.com/update.xml"
        );
    }

    #[test]
    fn chromium_json_contains_entry() {
        let j = chromium_managed_json(&ident());
        assert!(j.contains("ExtensionInstallForcelist"));
        assert!(j.contains(&ident().forcelist_entry()));
        // Must be valid JSON.
        let v: serde_json::Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["ExtensionInstallForcelist"][0], ident().forcelist_entry());
    }

    #[test]
    fn firefox_settings_object_is_valid_json() {
        // This exact string is written as a single Windows REG_SZ value.
        let obj = firefox_extension_settings(&ident()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&obj).unwrap();
        assert_eq!(
            v[FIREFOX_EXTENSION_ID]["install_url"],
            "https://example.com/cg.xpi"
        );
    }

    #[test]
    fn firefox_json_present_only_with_url() {
        let with = firefox_policies_json(&ident()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&with).unwrap();
        assert_eq!(
            v["policies"]["ExtensionSettings"][FIREFOX_EXTENSION_ID]["installation_mode"],
            "force_installed"
        );
        let mut no_ff = ident();
        no_ff.firefox_install_url.clear();
        assert!(firefox_policies_json(&no_ff).is_none());
    }
}
