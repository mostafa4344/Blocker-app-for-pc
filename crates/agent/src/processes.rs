//! Desktop app blocking: finds running processes that match the active policy
//! and closes them. Also reports which apps are running for daily allowances.

use std::collections::HashSet;

use focusfort_core::matcher::normalize_app;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

/// Processes that are never closed, whatever the block lists say, because
/// closing them would break the operating system or FocusFort itself.
const NEVER_CLOSE: &[&str] = &[
    "focusfort-agent",
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

pub fn browser_family(name: &str) -> Option<BrowserFamily> {
    match normalize_app(name).as_str() {
        "chrome" | "google chrome" | "msedge" | "microsoft edge" | "brave" | "brave browser"
        | "opera" | "opera_gx" | "vivaldi" | "chromium" | "chromium-browser" | "yandex"
        | "browser" | "arc" => Some(BrowserFamily::Chromium),
        "firefox" | "firefox-esr" | "librewolf" | "waterfox" | "zen" | "floorp" => {
            Some(BrowserFamily::Firefox)
        }
        _ => None,
    }
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
}
