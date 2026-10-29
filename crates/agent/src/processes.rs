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
/// Older extension versions only report the engine family ("chromium"). Such
/// a report can't say which browser sent it, so it only counts when this is
/// the only browser brand of that family running: otherwise an extension in
/// Chrome would make Edge (with no extension) look protected.
pub fn report_covers(
    seen_key: &str,
    brand: &str,
    family: BrowserFamily,
    sole_brand_of_family: bool,
) -> bool {
    seen_key == brand || (seen_key == family.key() && sole_brand_of_family)
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
    /// The program's original file name from its version information, when
    /// the file was renamed (e.g. `chrome.exe` copied as `notes.exe`).
    pub original: Option<String>,
    /// Seconds since the Unix epoch.
    pub start_time: u64,
}

pub struct Watcher {
    sys: System,
    seen_names: HashSet<String>,
    own_pid: Option<Pid>,
    /// Program path -> whether it is a Tor Browser binary (cached).
    tor_paths: std::collections::HashMap<std::path::PathBuf, bool>,
    /// Program path -> original file name from its version information (cached).
    originals: std::collections::HashMap<std::path::PathBuf, Option<String>>,
}

/// What a renamed program really is: browsers are recognised by their
/// original file name, so renaming `chrome.exe` doesn't dodge browser rules.
pub fn effective_name(actual: &str, original: Option<&str>) -> String {
    match original {
        Some(o)
            if browser_support(actual) == BrowserSupport::NotBrowser
                && browser_support(o) != BrowserSupport::NotBrowser =>
        {
            o.to_string()
        }
        _ => actual.to_string(),
    }
}

/// `OriginalFilename` from a Windows program's version information.
#[cfg(windows)]
fn original_filename(path: &std::path::Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "version")]
    extern "system" {
        fn GetFileVersionInfoSizeW(file: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(file: *const u16, handle: u32, len: u32, data: *mut u8) -> i32;
        fn VerQueryValueW(
            block: *const u8,
            sub: *const u16,
            buf: *mut *mut u8,
            len: *mut u32,
        ) -> i32;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: Win32 calls with buffers that stay alive for the whole block;
    // returned pointers point into `data` and are read within its length.
    unsafe {
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut handle);
        if size == 0 || size > 4 << 20 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, data.as_mut_ptr()) == 0 {
            return None;
        }
        let query = |sub: &str| -> Option<(*mut u8, u32)> {
            let w: Vec<u16> = sub.encode_utf16().chain(Some(0)).collect();
            let mut ptr: *mut u8 = std::ptr::null_mut();
            let mut len = 0u32;
            (VerQueryValueW(data.as_ptr(), w.as_ptr(), &mut ptr, &mut len) != 0
                && !ptr.is_null()
                && len > 0)
                .then_some((ptr, len))
        };
        let mut langs: Vec<(u16, u16)> = Vec::new();
        if let Some((ptr, len)) = query(r"\VarFileInfo\Translation") {
            let pairs = std::slice::from_raw_parts(ptr as *const u16, (len / 2) as usize);
            langs.extend(pairs.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])));
        }
        langs.extend([(0x0409, 0x04b0), (0x0409, 0x04e4), (0x0000, 0x04b0)]);
        for (lang, cp) in langs {
            if let Some((ptr, len)) = query(&format!(
                r"\StringFileInfo\{lang:04x}{cp:04x}\OriginalFilename"
            )) {
                let chars = std::slice::from_raw_parts(ptr as *const u16, len as usize);
                let name = String::from_utf16_lossy(chars)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }
}

#[cfg(not(windows))]
fn original_filename(_path: &std::path::Path) -> Option<String> {
    None
}

/// Name reported for a Tor Browser process, whatever its file is called.
pub const TOR_BROWSER_NAME: &str = "Tor Browser";

/// Tor Browser runs as `firefox.exe` (and can be renamed), so it is recognised
/// by its layout instead: the Tor daemon ships next to the browser at
/// `TorBrowser/Tor/tor(.exe)`. Renaming the folder doesn't change that.
pub fn is_tor_browser_dir(dir: &std::path::Path) -> bool {
    let tor = dir.join("TorBrowser").join("Tor");
    tor.join("tor.exe").is_file() || tor.join("tor").is_file()
}

impl Watcher {
    pub fn new() -> Self {
        Watcher {
            sys: System::new(),
            seen_names: HashSet::new(),
            own_pid: sysinfo::get_current_pid().ok(),
            tor_paths: std::collections::HashMap::new(),
            originals: std::collections::HashMap::new(),
        }
    }

    /// Current processes plus the set of normalised names that were not running at the previous scan.
    pub fn scan(&mut self) -> (Vec<Proc>, HashSet<String>) {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        let tor_paths = &mut self.tor_paths;
        let originals = &mut self.originals;
        let procs: Vec<Proc> = self
            .sys
            .processes()
            .iter()
            .filter(|(pid, _)| Some(**pid) != self.own_pid)
            .map(|(pid, p)| {
                let is_tor = p.exe().is_some_and(|exe| {
                    *tor_paths
                        .entry(exe.to_path_buf())
                        .or_insert_with(|| exe.parent().is_some_and(is_tor_browser_dir))
                });
                let actual = p.name().to_string_lossy().into_owned();
                let original = p
                    .exe()
                    .and_then(|exe| {
                        originals
                            .entry(exe.to_path_buf())
                            .or_insert_with(|| original_filename(exe))
                            .clone()
                    })
                    .filter(|o| normalize_app(o) != normalize_app(&actual));
                Proc {
                    pid: *pid,
                    name: if is_tor {
                        TOR_BROWSER_NAME.to_string()
                    } else {
                        effective_name(&actual, original.as_deref())
                    },
                    original,
                    start_time: p.start_time(),
                }
            })
            .collect();
        // Forget programs that are no longer running so the cache stays small.
        if self.tor_paths.len() > 4096 {
            self.tor_paths.clear();
        }
        if self.originals.len() > 4096 {
            self.originals.clear();
        }
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

    #[cfg(windows)]
    #[test]
    fn original_name_survives_a_rename() {
        let src = std::path::Path::new(
            r"C:\Windows\System32
otepad.exe",
        );
        if !src.is_file() {
            return;
        }
        let copy = std::env::temp_dir().join(format!("cg-renamed-{}.exe", std::process::id()));
        std::fs::copy(src, &copy).unwrap();
        let original = original_filename(&copy).map(|o| normalize_app(&o));
        let _ = std::fs::remove_file(&copy);
        assert_eq!(original.as_deref(), Some("notepad"));
    }

    #[test]
    fn renamed_browsers_are_still_browsers() {
        assert_eq!(
            effective_name("notes.exe", Some("chrome.exe")),
            "chrome.exe"
        );
        assert_eq!(effective_name("x.exe", Some("tor.exe")), "tor.exe");
        // Real names win, and ordinary apps keep their own name.
        assert_eq!(
            effective_name("msedge.exe", Some("chrome.exe")),
            "msedge.exe"
        );
        assert_eq!(
            effective_name("Discord.exe", Some("electron.exe")),
            "Discord.exe"
        );
        assert_eq!(effective_name("notes.exe", None), "notes.exe");
    }

    #[test]
    fn tor_browser_is_found_by_its_layout_not_its_name() {
        let root = std::env::temp_dir().join(format!("cg-tor-test-{}", std::process::id()));
        let tor_dir = root.join("Renamed Folder").join("Browser");
        std::fs::create_dir_all(tor_dir.join("TorBrowser").join("Tor")).unwrap();
        std::fs::write(tor_dir.join("TorBrowser").join("Tor").join("tor.exe"), b"").unwrap();
        let firefox_dir = root.join("Mozilla Firefox");
        std::fs::create_dir_all(&firefox_dir).unwrap();
        assert!(is_tor_browser_dir(&tor_dir));
        assert!(!is_tor_browser_dir(&firefox_dir));
        // The name it is reported under is treated as an unsupported browser.
        assert_eq!(
            browser_support(TOR_BROWSER_NAME),
            BrowserSupport::Unsupported
        );
        assert!(browser_brand(TOR_BROWSER_NAME).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn brands_are_checked_per_browser() {
        let (chrome, _) = browser_brand("chrome.exe").unwrap();
        let (edge, _) = browser_brand("msedge.exe").unwrap();
        assert_eq!((chrome, edge), ("chrome", "edge"));
        // An extension in Chrome does not cover Edge...
        assert!(report_covers(
            "chrome",
            chrome,
            BrowserFamily::Chromium,
            false
        ));
        assert!(!report_covers(
            "chrome",
            edge,
            BrowserFamily::Chromium,
            true
        ));
        // ...an old family-only report covers a browser only when it is the
        // only Chromium browser running (otherwise we can't tell who sent it)...
        assert!(report_covers(
            "chromium",
            edge,
            BrowserFamily::Chromium,
            true
        ));
        assert!(!report_covers(
            "chromium",
            edge,
            BrowserFamily::Chromium,
            false
        ));
        // ...and never another family.
        assert!(!report_covers(
            "chromium",
            "firefox",
            BrowserFamily::Firefox,
            true
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
