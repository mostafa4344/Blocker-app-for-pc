//! Desktop app blocking: finds running processes that match the active policy
//! and closes them. Also reports which apps are running for daily allowances.

use std::collections::HashSet;

use contentguard_core::matcher::normalize_app;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

/// Processes that are never closed, whatever the block lists say, because
/// closing them would break the operating system or ContentGuard itself.
const NEVER_CLOSE: &[&str] = &[
    "contentguard-agent",
    "system",
    "idle",
    "smss",
    "csrss",
    "wininit",
    "winlogon",
    "services",
    "lsass",
    "svchost",
    "explorer",
    "dwm",
    "fontdrvhost",
    "sihost",
    "taskhostw",
    "ctfmon",
    "conhost",
    "runtimebroker",
    "searchhost",
    "startmenuexperiencehost",
    "shellexperiencehost",
    "taskmgr",
    "init",
    "systemd",
    "launchd",
    "kernel_task",
    "windowserver",
    "loginwindow",
    "finder",
    "dock",
    "xorg",
    "gnome-shell",
    "plasmashell",
    "kwin_x11",
    "kwin_wayland",
    "sddm",
    "gdm",
    "lightdm",
    "sshd",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserFamily {
    Chromium,
    Firefox,
}

impl BrowserFamily {
    pub fn key(self) -> &'static str {
        match self {
            BrowserFamily::Chromium => "chromium",
            BrowserFamily::Firefox => "firefox",
        }
    }
}

/// Supported browsers: the ContentGuard extension runs here, so website,
/// keyword and content (adult) scanning all work. Normalised executable names.
pub const SUPPORTED_BROWSERS: &[&str] = &[
    "chrome",
    "google chrome",
    "msedge",
    "microsoft edge",
    "brave",
    "brave browser",
    "opera",
    "opera_gx",
    "operagx",
    "vivaldi",
    "chromium",
    "chromium-browser",
    "yandex",
    "arc",
    "firefox",
    "firefox-esr",
    "librewolf",
    "waterfox",
    "zen",
    "floorp",
];

/// Known browsers where the extension cannot run (no compatible WebExtension
/// support, or a different extension model), so content scanning is impossible.
/// When "block unsupported browsers" is on, these are closed while blocking.
pub const UNSUPPORTED_BROWSERS: &[&str] = &[
    "safari",
    // Tor routes DNS through its own network, so the hosts file cannot filter it.
    "tor",
    "torbrowser",
    "tor browser",
    "palemoon",
    "basilisk",
    "seamonkey",
    "iexplore",
    "msie",
    "qutebrowser",
    "falkon",
    "konqueror",
    "epiphany",
    "gnome-web",
    "midori",
    "otter",
    "otter-browser",
    "dillo",
    "lynx",
    "w3m",
    "links",
    "elinks",
    "maxthon",
    "ucbrowser",
    "360se",
    "360chrome",
    "duckduckgo",
    "min",
    "colibri",
    "wexond",
    "netsurf",
    "surf",
    "badwolf",
];

/// How ContentGuard relates to a process that is a web browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSupport {
    /// Extension works; `BrowserFamily` says which store/engine.
    Supported(BrowserFamily),
    /// A known browser we cannot scan.
    Unsupported,
    /// Not a browser we recognise.
    NotBrowser,
}

pub fn browser_family(name: &str) -> Option<BrowserFamily> {
    match normalize_app(name).as_str() {
        "chrome" | "google chrome" | "msedge" | "microsoft edge" | "brave" | "brave browser"
        | "opera" | "opera_gx" | "operagx" | "vivaldi" | "chromium" | "chromium-browser"
        | "yandex" | "arc" => Some(BrowserFamily::Chromium),
        "firefox" | "firefox-esr" | "librewolf" | "waterfox" | "zen" | "floorp" => {
            Some(BrowserFamily::Firefox)
        }
        _ => None,
    }
}

/// Which browser product a process is, as `(report key, display name)`.
/// The report key is what the extension running in that browser sends
/// (see `extension/background.js`), so each browser is checked on its own:
/// an extension in Chrome does not make Edge count as protected.
/// Chromium browsers that don't identify themselves share the "generic" key.
pub fn browser_brand(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match normalize_app(name).as_str() {
        "chrome" | "google chrome" => ("chrome", "Chrome"),
        "msedge" | "microsoft edge" => ("edge", "Edge"),
        "brave" | "brave browser" => ("brave", "Brave"),
        "opera" | "opera_gx" | "operagx" => ("opera", "Opera"),
        "yandex" => ("yandex", "Yandex"),
        "vivaldi" => ("generic", "Vivaldi"),
        "arc" => ("generic", "Arc"),
        "chromium" | "chromium-browser" => ("generic", "Chromium"),
        "firefox" | "firefox-esr" => ("firefox", "Firefox"),
        "librewolf" => ("firefox", "LibreWolf"),
        "waterfox" => ("firefox", "Waterfox"),
        "zen" => ("firefox", "Zen"),
        "floorp" => ("firefox", "Floorp"),
        _ => return None,
    })
}

/// Report keys an extension may send.
pub const REPORT_KEYS: &[&str] = &[
    "chrome", "edge", "brave", "opera", "yandex", "generic", "firefox",
];

/// Whether an extension report under `seen_key` covers a browser process.
/// Older extension versions only report the engine family ("chromium"), which
/// still covers every Chromium browser until they update.
pub fn report_covers(seen_key: &str, brand: &str, family: BrowserFamily) -> bool {
    seen_key == brand || seen_key == family.key()
}

pub fn browser_support(name: &str) -> BrowserSupport {
    if let Some(f) = browser_family(name) {
        return BrowserSupport::Supported(f);
    }
    if UNSUPPORTED_BROWSERS.contains(&normalize_app(name).as_str()) {
        return BrowserSupport::Unsupported;
    }
    BrowserSupport::NotBrowser
}

pub fn is_protected(name: &str) -> bool {
    NEVER_CLOSE.contains(&normalize_app(name).as_str())
}

#[derive(Debug, Clone)]
pub struct Proc {
    pub pid: Pid,
    pub name: String,
    /// Seconds since the Unix epoch.
    pub start_time: u64,
}

pub struct Watcher {
    sys: System,
    seen_names: HashSet<String>,
    own_pid: Option<Pid>,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher {
            sys: System::new(),
            seen_names: HashSet::new(),
            own_pid: sysinfo::get_current_pid().ok(),
        }
    }

    /// Current processes plus the set of normalised names that were not running at the previous scan.
    pub fn scan(&mut self) -> (Vec<Proc>, HashSet<String>) {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        let procs: Vec<Proc> = self
            .sys
            .processes()
            .iter()
            .filter(|(pid, _)| Some(**pid) != self.own_pid)
            .map(|(pid, p)| Proc {
                pid: *pid,
                name: p.name().to_string_lossy().into_owned(),
                start_time: p.start_time(),
            })
            .collect();
        let names: HashSet<String> = procs.iter().map(|p| normalize_app(&p.name)).collect();
        let new_names = names.difference(&self.seen_names).cloned().collect();
        self.seen_names = names;
        (procs, new_names)
    }

    /// Asks the process to exit, forcing it if the platform has no polite signal.
    pub fn close(&self, pid: Pid) -> bool {
        let Some(p) = self.sys.process(pid) else {
            return false;
        };
        if is_protected(&p.name().to_string_lossy()) {
            return false;
        }
        p.kill_with(Signal::Term).unwrap_or_else(|| p.kill())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brands_are_checked_per_browser() {
        let (chrome, _) = browser_brand("chrome.exe").unwrap();
        let (edge, _) = browser_brand("msedge.exe").unwrap();
        assert_eq!((chrome, edge), ("chrome", "edge"));
        // An extension in Chrome does not cover Edge...
        assert!(report_covers("chrome", chrome, BrowserFamily::Chromium));
        assert!(!report_covers("chrome", edge, BrowserFamily::Chromium));
        // ...but an old family-only report still covers both.
        assert!(report_covers("chromium", edge, BrowserFamily::Chromium));
        assert!(!report_covers(
            "chromium",
            "firefox",
            BrowserFamily::Firefox
        ));
        assert_eq!(browser_brand("librewolf").unwrap().0, "firefox");
        assert!(browser_brand("notepad.exe").is_none());
        for (k, _) in [
            "chrome", "msedge", "brave", "opera", "yandex", "vivaldi", "firefox",
        ]
        .iter()
        .filter_map(|n| browser_brand(n))
        {
            assert!(REPORT_KEYS.contains(&k));
        }
    }

    #[test]
    fn families() {
        assert_eq!(browser_family("chrome.exe"), Some(BrowserFamily::Chromium));
        assert_eq!(browser_family("msedge.exe"), Some(BrowserFamily::Chromium));
        assert_eq!(browser_family("firefox"), Some(BrowserFamily::Firefox));
        assert_eq!(browser_family("notepad.exe"), None);
    }

    #[test]
    fn protected_names() {
        assert!(is_protected("explorer.exe"));
        assert!(is_protected("C:\\Windows\\System32\\svchost.exe"));
        assert!(!is_protected("steam.exe"));
    }

    #[test]
    fn support_classification() {
        assert_eq!(
            browser_support("chrome.exe"),
            BrowserSupport::Supported(BrowserFamily::Chromium)
        );
        assert_eq!(
            browser_support("Firefox"),
            BrowserSupport::Supported(BrowserFamily::Firefox)
        );
        assert_eq!(browser_support("Safari.app"), BrowserSupport::Unsupported);
        assert_eq!(browser_support("palemoon.exe"), BrowserSupport::Unsupported);
        assert_eq!(browser_support("notepad.exe"), BrowserSupport::NotBrowser);
    }
}
