//! Platform locations. Every path can be overridden with an environment
//! variable so the agent can run unprivileged during development and tests.

use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("FOCUSFORT_DATA_DIR") {
        return d.into();
    }
    if cfg!(windows) {
        let base = std::env::var_os("ProgramData").unwrap_or_else(|| r"C:\ProgramData".into());
        PathBuf::from(base).join("FocusFort")
    } else if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support/FocusFort")
    } else {
        PathBuf::from("/var/lib/focusfort")
    }
}

pub fn hosts_file() -> PathBuf {
    if let Some(p) = std::env::var_os("FOCUSFORT_HOSTS_FILE") {
        return p.into();
    }
    if cfg!(windows) {
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
        PathBuf::from(root).join(r"System32\drivers\etc\hosts")
    } else {
        PathBuf::from("/etc/hosts")
    }
}

pub fn listen_port() -> u16 {
    std::env::var("FOCUSFORT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(17345)
}
