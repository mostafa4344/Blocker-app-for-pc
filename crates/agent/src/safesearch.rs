//! Force SafeSearch / Restricted Mode via the hosts file — the method the big
//! engines officially support: point each search host at the IP of its "safe"
//! variant (Google's forcesafesearch, YouTube's restrict, Bing's strict,
//! DuckDuckGo's safe). Those variants serve certificates valid for the real
//! host, so HTTPS keeps working and no content is decrypted.

use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

/// (safe hostname whose IP we use, documented fallback IP, real hostnames to pin).
/// The fallback is used only when live DNS resolution fails; Google publishes
/// stable anycast IPs for its SafeSearch/Restricted endpoints.
const MAP: &[(&str, Option<&str>, &[&str])] = &[
    (
        "forcesafesearch.google.com",
        Some("216.239.38.120"),
        &[
            "google.com",
            "www.google.com",
            "google.co.uk",
            "google.ca",
            "google.com.au",
            "google.de",
            "google.fr",
            "google.es",
            "google.it",
            "google.nl",
            "google.co.in",
            "google.com.bd",
            "google.com.pk",
            "google.com.br",
            "google.co.id",
            "google.com.tr",
            "google.ru",
            "google.pl",
            "google.co.jp",
        ],
    ),
    (
        "restrict.youtube.com",
        Some("216.239.38.120"),
        &[
            "www.youtube.com",
            "m.youtube.com",
            "youtube.com",
            "youtubei.googleapis.com",
            "youtube.googleapis.com",
            "www.youtube-nocookie.com",
        ],
    ),
    (
        "strict.bing.com",
        None,
        &["www.bing.com", "bing.com", "cn.bing.com"],
    ),
    (
        "safe.duckduckgo.com",
        None,
        &["duckduckgo.com", "www.duckduckgo.com"],
    ),
];

/// Resolves a hostname to a single IPv4 string, preferring IPv4 for hosts-file use.
fn resolve_ipv4(host: &str) -> Option<String> {
    let addrs = (host, 443).to_socket_addrs().ok()?;
    let mut v6: Option<String> = None;
    for a in addrs {
        if a.is_ipv4() {
            return Some(a.ip().to_string());
        }
        v6.get_or_insert_with(|| a.ip().to_string());
    }
    v6
}

/// Builds `(ip, host)` redirect entries, skipping any host in `blocked` (a full
/// block takes precedence over SafeSearch). `resolve` is injected for testing.
pub fn build_with<F: Fn(&str) -> Option<String>>(
    blocked: &[String],
    resolve: F,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (safe_host, fallback, reals) in MAP {
        let Some(ip) = resolve(safe_host).or_else(|| fallback.map(String::from)) else {
            continue;
        };
        for real in *reals {
            if blocked.iter().any(|b| b.eq_ignore_ascii_case(real)) {
                continue;
            }
            out.push((ip.clone(), real.to_string()));
        }
    }
    out
}

/// Caches resolved SafeSearch entries and refreshes them periodically.
pub struct SafeSearch {
    cached: Vec<(String, String)>,
    last: Option<Instant>,
    last_blocked: Vec<String>,
}

const REFRESH: Duration = Duration::from_secs(6 * 3600);

impl SafeSearch {
    pub fn new() -> Self {
        SafeSearch {
            cached: Vec::new(),
            last: None,
            last_blocked: Vec::new(),
        }
    }

    /// Entries to apply now. Re-resolves every few hours, or when the blocked
    /// set changed (so a newly blocked engine stops getting a SafeSearch pin).
    pub fn entries(&mut self, enabled: bool, blocked: &[String]) -> &[(String, String)] {
        if !enabled {
            self.cached.clear();
            self.last = None;
            return &self.cached;
        }
        let stale = self.last.map(|t| t.elapsed() > REFRESH).unwrap_or(true);
        if stale || blocked != self.last_blocked.as_slice() {
            self.cached = build_with(blocked, resolve_ipv4);
            self.last = Some(Instant::now());
            self.last_blocked = blocked.to_vec();
        }
        &self.cached
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_entries_and_respects_blocks() {
        let resolve = |h: &str| match h {
            "forcesafesearch.google.com" => Some("216.239.38.120".to_string()),
            "restrict.youtube.com" => Some("216.239.38.119".to_string()),
            _ => None, // bing/ddg unresolved in this test
        };
        let entries = build_with(&["www.bing.com".into()], resolve);
        // Google + YouTube resolved; bing/ddg skipped (unresolved).
        assert!(entries
            .iter()
            .any(|(ip, h)| ip == "216.239.38.120" && h == "www.google.com"));
        assert!(entries.iter().any(|(_, h)| h == "www.youtube.com"));
        assert!(!entries.iter().any(|(_, h)| h.contains("bing")));
    }

    #[test]
    fn falls_back_when_dns_fails() {
        // No live resolution at all: Google & YouTube still covered via fallback IPs.
        let entries = build_with(&[], |_| None);
        assert!(entries
            .iter()
            .any(|(ip, h)| ip == "216.239.38.120" && h == "www.google.com"));
        assert!(entries.iter().any(|(_, h)| h == "www.youtube.com"));
        // Bing/DuckDuckGo have no fallback, so they are absent.
        assert!(!entries
            .iter()
            .any(|(_, h)| h.contains("bing") || h.contains("duckduckgo")));
    }

    #[test]
    fn blocked_host_is_not_pinned() {
        let resolve = |_: &str| Some("1.2.3.4".to_string());
        let entries = build_with(&["youtube.com".into(), "www.youtube.com".into()], resolve);
        assert!(!entries.iter().any(|(_, h)| h == "www.youtube.com"));
        // A different youtube host not in the block list is still pinned.
        assert!(entries.iter().any(|(_, h)| h == "m.youtube.com"));
    }
}
