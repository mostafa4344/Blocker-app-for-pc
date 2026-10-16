//! Optional DNS family-filter layer: point the OS resolver at a filtering DNS
//! service so adult/malware domains are blocked network-wide (every browser and
//! app), before any connection is made. This is a well-established, OS-sanctioned
//! technique (no decryption, no driver), used by OpenDNS FamilyShield etc.
//!
//! DEFAULT OFF and opt-in. The command builders below are pure and unit-tested;
//! actually changing the system resolver is OS-specific and MUST be verified on
//! a real machine before relying on it (see docs/SHIP-CHECKLIST.md). We always
//! keep the ability to restore the previous DNS so networking is never left
//! broken.
//
// The command builders are complete and unit-tested now; the execution wiring
// (enumerate interfaces, apply, save & restore original DNS) is added together
// with real-machine testing, so some items here are not referenced yet.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DnsFilter {
    /// Do not touch system DNS.
    #[default]
    Off,
    /// Cloudflare for Families: blocks malware + adult content.
    Family,
    /// CleanBrowsing Family: stricter (adult + proxies/VPN + mixed content).
    Strict,
}

impl DnsFilter {
    pub fn parse(s: &str) -> DnsFilter {
        match s.trim().to_ascii_lowercase().as_str() {
            "family" => DnsFilter::Family,
            "strict" => DnsFilter::Strict,
            _ => DnsFilter::Off,
        }
    }

    /// IPv4 resolvers for this level (primary, secondary).
    pub fn ipv4(self) -> &'static [&'static str] {
        match self {
            DnsFilter::Off => &[],
            DnsFilter::Family => &["1.1.1.3", "1.0.0.3"],
            DnsFilter::Strict => &["185.228.168.168", "185.228.169.168"],
        }
    }
    /// IPv6 resolvers for this level.
    pub fn ipv6(self) -> &'static [&'static str] {
        match self {
            DnsFilter::Off => &[],
            DnsFilter::Family => &["2606:4700:4700::1113", "2606:4700:4700::1003"],
            DnsFilter::Strict => &["2a0d:2a00:1::", "2a0d:2a00:2::"],
        }
    }
}

// ---- pure command builders (unit-tested; execution wrappers call these) ----

/// Windows: argv lists to pin an interface's DNS to `ips` (IPv4).
/// `netsh interface ipv4 set dnsservers name="IF" static IP primary`
/// then `add dnsservers name="IF" IP index=N` for the rest.
pub fn windows_set_argv(iface: &str, ips: &[&str]) -> Vec<Vec<String>> {
    let mut cmds = Vec::new();
    for (i, ip) in ips.iter().enumerate() {
        if i == 0 {
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "set".into(),
                "dnsservers".into(),
                format!("name={iface}"),
                "static".into(),
                ip.to_string(),
                "primary".into(),
            ]);
        } else {
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "add".into(),
                "dnsservers".into(),
                format!("name={iface}"),
                ip.to_string(),
                format!("index={}", i + 1),
            ]);
        }
    }
    cmds
}

/// Windows: argv to hand an interface's DNS back to DHCP.
pub fn windows_reset_argv(iface: &str) -> Vec<String> {
    vec![
        "interface".into(),
        "ipv4".into(),
        "set".into(),
        "dnsservers".into(),
        format!("name={iface}"),
        "dhcp".into(),
    ]
}

/// macOS: `networksetup -setdnsservers <service> ip1 ip2 ...` (or `empty` to clear).
pub fn macos_set_argv(service: &str, ips: &[&str]) -> Vec<String> {
    let mut v = vec!["-setdnsservers".to_string(), service.to_string()];
    if ips.is_empty() {
        v.push("empty".into());
    } else {
        v.extend(ips.iter().map(|s| s.to_string()));
    }
    v
}

/// Linux (resolv.conf fallback): the file body for `ips`.
pub fn resolv_conf(ips: &[&str]) -> String {
    let mut s = String::from("# Managed by ContentGuard\n");
    for ip in ips {
        s.push_str(&format!("nameserver {ip}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_have_resolvers() {
        assert!(DnsFilter::Off.ipv4().is_empty());
        assert_eq!(DnsFilter::Family.ipv4(), &["1.1.1.3", "1.0.0.3"]);
        assert_eq!(DnsFilter::Strict.ipv4()[0], "185.228.168.168");
        assert_eq!(DnsFilter::Family.ipv6().len(), 2);
    }

    #[test]
    fn windows_argv_shape() {
        let cmds = windows_set_argv("Wi-Fi", &["1.1.1.3", "1.0.0.3"]);
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains(&"static".to_string()));
        assert!(cmds[0].contains(&"name=Wi-Fi".to_string()));
        assert!(cmds[0].contains(&"primary".to_string()));
        assert!(cmds[1].contains(&"index=2".to_string()));
        assert_eq!(windows_reset_argv("Wi-Fi").last().unwrap(), "dhcp");
    }

    #[test]
    fn macos_and_linux_builders() {
        assert_eq!(
            macos_set_argv("Wi-Fi", &["1.1.1.3"]),
            vec!["-setdnsservers", "Wi-Fi", "1.1.1.3"]
        );
        assert_eq!(
            macos_set_argv("Wi-Fi", &[]),
            vec!["-setdnsservers", "Wi-Fi", "empty"]
        );
        assert!(resolv_conf(&["1.1.1.3", "1.0.0.3"]).contains("nameserver 1.1.1.3"));
    }
}
