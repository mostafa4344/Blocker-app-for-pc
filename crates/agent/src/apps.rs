//! Programs installed on this computer, for the "choose an app" picker.
//!
//! Blocking works on the program's file name (e.g. `EpicGamesLauncher.exe`),
//! which people don't know. The Start-menu shortcuts give both the friendly
//! name and the real program, so the picker can show "Epic Games Launcher"
//! and block the right file. Shortcuts are read with a small parser for the
//! documented Shell Link (.lnk) format [MS-SHLLINK].

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InstalledApp {
    /// Friendly name (the shortcut's name), e.g. "Discord".
    pub name: String,
    /// Program file name used for blocking, e.g. "discord.exe".
    pub exe: String,
}

const HAS_ID_LIST: u32 = 0x1;
const HAS_LINK_INFO: u32 = 0x2;
const HAS_NAME: u32 = 0x4;
const HAS_RELATIVE_PATH: u32 = 0x8;
const HAS_WORKING_DIR: u32 = 0x10;
const HAS_ARGUMENTS: u32 = 0x20;
const IS_UNICODE: u32 = 0x80;

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
fn cstr_at(b: &[u8], at: usize) -> Option<String> {
    let rest = b.get(at..)?;
    let end = rest.iter().position(|&c| c == 0)?;
    Some(String::from_utf8_lossy(&rest[..end]).into_owned())
}
fn wstr_at(b: &[u8], at: usize) -> Option<String> {
    let mut units = Vec::new();
    let mut i = at;
    loop {
        let u = u16_at(b, i)?;
        if u == 0 {
            break;
        }
        units.push(u);
        i += 2;
    }
    Some(String::from_utf16_lossy(&units))
}

/// The program a shortcut points to: the full local path when the shortcut
/// stores one, otherwise its relative path. `None` for anything unreadable.
#[cfg(test)]
fn lnk_target(b: &[u8]) -> Option<String> {
    lnk_parts(b).map(|(t, _)| t)
}

/// `(target, arguments)` of a shortcut.
pub fn lnk_parts(b: &[u8]) -> Option<(String, String)> {
    if u32_at(b, 0)? != 0x4C {
        return None;
    }
    let flags = u32_at(b, 0x14)?;
    let mut pos = 0x4C;
    if flags & HAS_ID_LIST != 0 {
        pos += 2 + u16_at(b, pos)? as usize;
    }
    let mut local: Option<String> = None;
    if flags & HAS_LINK_INFO != 0 {
        let info = pos;
        let size = u32_at(b, info)? as usize;
        let header = u32_at(b, info + 4)?;
        let info_flags = u32_at(b, info + 8)?;
        if info_flags & 1 != 0 {
            local = if header >= 0x24 {
                let off = u32_at(b, info + 0x1C)? as usize;
                wstr_at(b, info + off)
            } else {
                let off = u32_at(b, info + 0x10)? as usize;
                cstr_at(b, info + off)
            };
        }
        pos += size;
    }
    // StringData: NAME, RELATIVE_PATH, WORKING_DIR, ARGUMENTS (count + characters).
    let unicode = flags & IS_UNICODE != 0;
    let mut field = |flag: u32| -> Option<String> {
        if flags & flag == 0 {
            return Some(String::new());
        }
        let n = u16_at(b, pos)? as usize;
        pos += 2;
        let s = if unicode {
            let units: Vec<u16> = (0..n)
                .map(|i| u16_at(b, pos + i * 2))
                .collect::<Option<_>>()?;
            pos += n * 2;
            String::from_utf16_lossy(&units)
        } else {
            let s = String::from_utf8_lossy(b.get(pos..pos + n)?).into_owned();
            pos += n;
            s
        };
        Some(s)
    };
    let _name = field(HAS_NAME);
    let relative = field(HAS_RELATIVE_PATH).unwrap_or_default();
    let _dir = field(HAS_WORKING_DIR);
    let args = field(HAS_ARGUMENTS).unwrap_or_default();
    let target = local.filter(|p| !p.is_empty()).unwrap_or(relative);
    (!target.is_empty()).then_some((target, args))
}

fn file_name(path: &str) -> String {
    path.rsplit(['\\', '/'])
        .next()
        .unwrap_or(path)
        .to_ascii_lowercase()
}

/// The program that really runs. Apps installed with Squirrel (Discord,
/// Slack...) start through `Update.exe --processStart X.exe`; blocking
/// Update.exe would block nothing, so the real program is used.
fn real_program(target: &str, args: &str) -> String {
    let exe = file_name(target);
    if exe == "update.exe" {
        let mut words = args.split_whitespace();
        while let Some(w) = words.next() {
            if w.eq_ignore_ascii_case("--processStart") {
                if let Some(real) = words.next() {
                    return file_name(real.trim_matches('"'));
                }
            }
        }
    }
    exe
}

/// Whether a shortcut is worth offering: a real program that isn't part of
/// Windows itself, an uninstaller, or this app.
fn worth_offering(name: &str, exe_path: &str) -> bool {
    let n = name.to_ascii_lowercase();
    let p = exe_path.to_ascii_lowercase().replace('/', "\\");
    let exe = file_name(&p);
    p.ends_with(".exe")
        && !p.contains("\\windows\\")
        && !p.contains("contentguard")
        && !n.contains("uninstall")
        && !n.contains("readme")
        && !n.contains("help")
        // Web apps installed from a browser run inside the browser; blocking
        // the proxy file would block nothing (block the website instead).
        && !matches!(exe.as_str(), "chrome_proxy.exe" | "msedge_proxy.exe" | "private_browsing.exe")
}

/// Background helpers and console tools that show up as running processes
/// but aren't apps anyone wants to block.
fn is_background_tool(exe: &str) -> bool {
    const TOOLS: &[&str] = &[
        "bash.exe",
        "sh.exe",
        "cmd.exe",
        "pwsh.exe",
        "powershell.exe",
        "conhost.exe",
        "cargo.exe",
        "rustup.exe",
        "rustc.exe",
        "curl.exe",
        "node.exe",
        "python.exe",
        "pythonw.exe",
        "git.exe",
        "ssh.exe",
        "msedgewebview2.exe",
        "chrome_proxy.exe",
        "msedge_proxy.exe",
    ];
    let words = [
        "helper",
        "service",
        "native-host",
        "crashpad",
        "update",
        "installer",
        "elevation",
    ];
    TOOLS.contains(&exe) || words.iter().any(|w| exe.contains(w))
}

/// Folders where apps people use are installed (not system or tool folders).
fn is_app_folder(path_lower: &str) -> bool {
    [
        "\\program files",
        "\\appdata\\local\\",
        "\\appdata\\roaming\\",
        "\\windowsapps\\",
    ]
    .iter()
    .any(|d| path_lower.contains(d))
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<InstalledApp>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            if depth < 4 {
                walk(&path, depth + 1, out);
            }
            continue;
        }
        if !path
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("lnk"))
        {
            continue;
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some((target, args)) = std::fs::read(&path).ok().and_then(|b| lnk_parts(&b)) else {
            continue;
        };
        if worth_offering(&name, &target) {
            let exe = real_program(&target, &args);
            if !out.iter().any(|a| a.exe == exe) {
                out.push(InstalledApp { name, exe });
            }
        }
    }
}

/// Start-menu folders: all users, plus every user profile (the agent runs as
/// a service, so "the current user" is not the person at the keyboard).
fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(pd) = std::env::var_os("ProgramData") {
        dirs.push(PathBuf::from(pd).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    let users = std::env::var_os("SystemDrive")
        .map(|d| PathBuf::from(format!("{}\\Users", d.to_string_lossy())))
        .unwrap_or_else(|| PathBuf::from(r"C:\Users"));
    if let Ok(entries) = std::fs::read_dir(users) {
        for e in entries.flatten() {
            dirs.push(
                e.path()
                    .join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs"),
            );
        }
    }
    dirs
}

/// Programs running now that aren't part of Windows (e.g. a Store app that
/// has no Start-menu shortcut file), as `(display name, program file)`.
pub fn running() -> Vec<InstalledApp> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let mut out: Vec<InstalledApp> = Vec::new();
    for p in sys.processes().values() {
        let Some(path) = p.exe().map(|e| e.to_string_lossy().into_owned()) else {
            continue;
        };
        let exe = file_name(&path);
        let lower = path.to_ascii_lowercase();
        if !exe.ends_with(".exe")
            || lower.contains("\\windows\\")
            || lower.contains("contentguard")
            || !is_app_folder(&lower)
            || is_background_tool(&exe)
            || crate::processes::is_protected(&exe)
            || out.iter().any(|a| a.exe == exe)
        {
            continue;
        }
        let name = exe.trim_end_matches(".exe").to_string();
        out.push(InstalledApp { name, exe });
    }
    out.sort_by_key(|a| a.name.clone());
    out
}

/// Installed programs, sorted by name.
pub fn installed() -> Vec<InstalledApp> {
    let mut out = Vec::new();
    if cfg!(windows) {
        for d in start_menu_dirs() {
            walk(&d, 0, &mut out);
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal shortcut: header + LinkInfo with an ANSI local path.
    fn lnk_with_local_path(path: &str) -> Vec<u8> {
        let mut b = vec![0u8; 0x4C];
        b[0..4].copy_from_slice(&0x4Cu32.to_le_bytes());
        b[0x14..0x18].copy_from_slice(&HAS_LINK_INFO.to_le_bytes());
        let header = 0x1Cu32;
        let size = header as usize + path.len() + 1;
        let mut info = vec![0u8; header as usize];
        info[0..4].copy_from_slice(&(size as u32).to_le_bytes());
        info[4..8].copy_from_slice(&header.to_le_bytes());
        info[8..12].copy_from_slice(&1u32.to_le_bytes());
        info[0x10..0x14].copy_from_slice(&header.to_le_bytes());
        info.extend_from_slice(path.as_bytes());
        info.push(0);
        b.extend(info);
        b
    }

    /// A shortcut with only Unicode string data: relative path + arguments.
    fn lnk_with_relative_path(rel: &str, args: &str) -> Vec<u8> {
        let mut b = vec![0u8; 0x4C];
        b[0..4].copy_from_slice(&0x4Cu32.to_le_bytes());
        b[0x14..0x18]
            .copy_from_slice(&(HAS_RELATIVE_PATH | HAS_ARGUMENTS | IS_UNICODE).to_le_bytes());
        for s in [rel, args] {
            let units: Vec<u16> = s.encode_utf16().collect();
            b.extend((units.len() as u16).to_le_bytes());
            for u in units {
                b.extend(u.to_le_bytes());
            }
        }
        b
    }

    #[test]
    fn reads_shortcut_targets() {
        let path = r"C:\Program Files\Epic Games\Launcher\EpicGamesLauncher.exe";
        assert_eq!(
            lnk_target(&lnk_with_local_path(path)).as_deref(),
            Some(path)
        );
        let (t, a) = lnk_parts(&lnk_with_relative_path(
            r"..\..\Discord\Update.exe",
            "--processStart Discord.exe",
        ))
        .unwrap();
        assert_eq!(t, r"..\..\Discord\Update.exe");
        assert_eq!(a, "--processStart Discord.exe");
        assert_eq!(lnk_target(b"not a shortcut"), None);
        assert_eq!(lnk_target(&[0x4C, 0, 0, 0]), None);
    }

    #[test]
    fn finds_the_program_that_really_runs() {
        assert_eq!(
            real_program(
                r"C:\Users\u\AppData\Local\Discord\Update.exe",
                "--processStart Discord.exe"
            ),
            "discord.exe"
        );
        assert_eq!(
            real_program(r"C:\X\Update.exe", r#"--processStart "Slack.exe""#),
            "slack.exe"
        );
        assert_eq!(real_program(r"C:\Steam\Steam.exe", ""), "steam.exe");
    }

    #[test]
    fn offers_only_real_programs() {
        assert!(worth_offering(
            "Steam",
            r"C:\Program Files (x86)\Steam\steam.exe"
        ));
        assert!(!worth_offering(
            "Notepad",
            r"C:\Windows\System32\notepad.exe"
        ));
        assert!(!worth_offering(
            "Uninstall Steam",
            r"C:\Program Files (x86)\Steam\uninstall.exe"
        ));
        assert!(!worth_offering("Manual", r"C:\Program Files\X\manual.pdf"));
        assert!(!worth_offering(
            "Claude",
            r"C:\Program Files\Google\Chrome\Application\chrome_proxy.exe"
        ));
        assert!(is_background_tool("cargo.exe") && is_background_tool("googleupdate.exe"));
        assert!(!is_background_tool("discord.exe"));
        assert!(is_app_folder(
            r"c:\users\u\appdata\local\discord\app-1.0\discord.exe"
        ));
        assert!(!is_app_folder(r"c:\tools\bin\x.exe"));
    }
}
