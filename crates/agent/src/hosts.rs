//! Website blocking through a clearly marked section of the system hosts file.
//!
//! The hosts file can only block exact host names, so for every domain rule a
//! few common subdomains are added as well. Path rules (`youtube.com/shorts`),
//! keywords, wildcards and allowlist mode are enforced by the browser extension.

use std::fs;
use std::io;
use std::path::Path;

use focusfort_core::matcher::SiteRule;

const BEGIN: &str = "# BEGIN FocusFort (managed automatically - do not edit)";
const END: &str = "# END FocusFort";
const SUBDOMAINS: &[&str] = &["", "www.", "m.", "mobile.", "web.", "touch.", "mbasic.", "old.", "new."];

/// Host names to block for the given site rules.
pub fn hosts_for(sites: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for rule in sites.iter().filter_map(|s| SiteRule::parse(s)) {
        // Path rules must not block the whole domain; wildcard-all is extension-only.
        if rule.path_prefix.is_some() || rule.domain.is_empty() {
            continue;
        }
        let subs: &[&str] = if rule.subdomains_only { &SUBDOMAINS[1..] } else { SUBDOMAINS };
        for sub in subs {
            let h = format!("{sub}{}", rule.domain);
            if !out.contains(&h) {
                out.push(h);
            }
        }
    }
    out
}

/// Returns `content` with the managed section replaced by entries for `hosts`
/// (or removed when `hosts` is empty).
pub fn render(content: &str, hosts: &[String]) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in content.lines() {
        if line.trim() == BEGIN {
            inside = true;
            continue;
        }
        if inside {
            if line.trim() == END {
                inside = false;
            }
            continue;
        }
        kept.push(line);
    }
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    let nl = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = kept.join(nl);
    if !out.is_empty() {
        out.push_str(nl);
    }
    if !hosts.is_empty() {
        out.push_str(nl);
        out.push_str(BEGIN);
        out.push_str(nl);
        for h in hosts {
            out.push_str(&format!("0.0.0.0 {h}{nl}:: {h}{nl}"));
        }
        out.push_str(END);
        out.push_str(nl);
    }
    out
}

/// Writes the managed section if it differs. Returns whether the file changed.
pub fn apply(path: &Path, hosts: &[String]) -> io::Result<bool> {
    let current = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let next = render(&current, hosts);
    if next == current {
        return Ok(false);
    }
    fs::write(path, next)?;
    flush_dns_cache();
    Ok(true)
}

fn flush_dns_cache() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("ipconfig").arg("/flushdns").creation_flags(CREATE_NO_WINDOW).output();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("dscacheutil").arg("-flushcache").output();
        let _ = std::process::Command::new("killall").args(["-HUP", "mDNSResponder"]).output();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("resolvectl").arg("flush-caches").output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_expansion() {
        let h = hosts_for(&["facebook.com".into(), "youtube.com/shorts".into(), "*.reddit.com".into(), "*".into()]);
        assert!(h.contains(&"facebook.com".to_string()));
        assert!(h.contains(&"www.facebook.com".to_string()));
        assert!(h.contains(&"old.reddit.com".to_string()));
        assert!(!h.contains(&"reddit.com".to_string()));
        assert!(!h.iter().any(|x| x.contains("youtube")));
    }

    #[test]
    fn render_preserves_user_lines() {
        let original = "127.0.0.1 localhost\n# my entry\n10.0.0.1 nas\n";
        let with = render(original, &["a.com".into()]);
        assert!(with.starts_with(original));
        assert!(with.contains("0.0.0.0 a.com"));
        let again = render(&with, &["a.com".into()]);
        assert_eq!(again, with);
        let removed = render(&with, &[]);
        assert_eq!(removed, original);
    }

    #[test]
    fn render_keeps_crlf() {
        let original = "127.0.0.1 localhost\r\n";
        let with = render(original, &["a.com".into()]);
        assert!(with.contains("0.0.0.0 a.com\r\n"));
        assert_eq!(render(&with, &[]), original);
    }
}
