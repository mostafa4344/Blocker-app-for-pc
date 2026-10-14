//! Matching of URLs, domains and executable names against rules.

/// Extracts the lower-case host from a URL or bare domain, dropping scheme,
/// credentials, port and path.
pub fn host_of(url: &str) -> String {
    let s = url.trim().to_ascii_lowercase();
    let s = s.split_once("://").map(|(_, rest)| rest).unwrap_or(&s);
    let s = s.split(['/', '?', '#']).next().unwrap_or("");
    let s = s.rsplit_once('@').map(|(_, h)| h).unwrap_or(s);
    let s = if s.starts_with('[') {
        s
    } else {
        s.split(':').next().unwrap_or("")
    };
    s.trim_end_matches('.').to_string()
}

/// Path (with query) of a URL, lower-case, always starting with `/`.
pub fn path_of(url: &str) -> String {
    let s = url.trim().to_ascii_lowercase();
    let s = s.split_once("://").map(|(_, rest)| rest).unwrap_or(&s);
    match s.find(['/', '?']) {
        Some(i) if s[i..].starts_with('/') => s[i..].split('#').next().unwrap_or("/").to_string(),
        Some(i) => format!("/{}", &s[i..]),
        None => "/".into(),
    }
}

/// A parsed site rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteRule {
    /// Domain without leading `*.`; empty means "every site".
    pub domain: String,
    /// `*.example.com` matches only subdomains, not `example.com` itself.
    pub subdomains_only: bool,
    /// Optional path prefix, e.g. `/shorts`.
    pub path_prefix: Option<String>,
}

impl SiteRule {
    pub fn parse(rule: &str) -> Option<SiteRule> {
        let r = rule.trim().to_ascii_lowercase();
        if r.is_empty() {
            return None;
        }
        if r == "*" {
            return Some(SiteRule { domain: String::new(), subdomains_only: false, path_prefix: None });
        }
        let r = r.split_once("://").map(|(_, rest)| rest.to_string()).unwrap_or(r);
        let (host_part, path) = match r.find('/') {
            Some(i) => (&r[..i], Some(r[i..].trim_end_matches('*').to_string())),
            None => (r.as_str(), None),
        };
        let (domain, subdomains_only) = match host_part.strip_prefix("*.") {
            Some(d) => (d, true),
            None => (host_part, false),
        };
        let domain = domain.trim_start_matches("www.").trim_end_matches('.');
        if domain.is_empty() || domain.contains(char::is_whitespace) {
            return None;
        }
        let path_prefix = path.filter(|p| p != "/" && !p.is_empty());
        Some(SiteRule { domain: domain.to_string(), subdomains_only, path_prefix })
    }

    pub fn matches_host(&self, host: &str) -> bool {
        if self.domain.is_empty() {
            return true;
        }
        let is_sub = host.len() > self.domain.len()
            && host.ends_with(&self.domain)
            && host.as_bytes()[host.len() - self.domain.len() - 1] == b'.';
        if self.subdomains_only {
            is_sub
        } else {
            host == self.domain || is_sub
        }
    }

    pub fn matches_url(&self, url: &str) -> bool {
        if !self.matches_host(&host_of(url)) {
            return false;
        }
        match &self.path_prefix {
            None => true,
            Some(p) => path_of(url).starts_with(p.as_str()),
        }
    }
}

pub fn site_matches(rules: &[String], url: &str) -> bool {
    rules.iter().filter_map(|r| SiteRule::parse(r)).any(|r| r.matches_url(url))
}

pub fn keyword_matches(keywords: &[String], url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    keywords
        .iter()
        .map(|k| k.trim().to_ascii_lowercase())
        .any(|k| !k.is_empty() && u.contains(&k))
}

/// Normalises an executable name or path: `C:\Games\Steam.EXE` -> `steam`.
pub fn normalize_app(name: &str) -> String {
    let base = name.trim().rsplit(['/', '\\']).next().unwrap_or("").to_lowercase();
    let base = base.strip_suffix(".exe").unwrap_or(&base);
    let base = base.strip_suffix(".app").unwrap_or(base);
    base.to_string()
}

pub fn app_matches(rules: &[String], process_name: &str) -> bool {
    let p = normalize_app(process_name);
    !p.is_empty() && rules.iter().any(|r| normalize_app(r) == p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_extraction() {
        assert_eq!(host_of("https://User@WWW.Facebook.com:443/x?y"), "www.facebook.com");
        assert_eq!(host_of("reddit.com"), "reddit.com");
        assert_eq!(host_of("http://example.com."), "example.com");
    }

    #[test]
    fn domain_rules() {
        let r = SiteRule::parse("facebook.com").unwrap();
        assert!(r.matches_url("https://www.facebook.com/feed"));
        assert!(r.matches_url("https://m.facebook.com"));
        assert!(r.matches_url("facebook.com"));
        assert!(!r.matches_url("https://notfacebook.com"));

        let w = SiteRule::parse("*.reddit.com").unwrap();
        assert!(w.matches_url("https://old.reddit.com"));
        assert!(!w.matches_url("https://reddit.com"));

        let www = SiteRule::parse("https://www.x.com/").unwrap();
        assert!(www.matches_url("x.com"));
    }

    #[test]
    fn path_rules() {
        let r = SiteRule::parse("youtube.com/shorts").unwrap();
        assert!(r.matches_url("https://www.youtube.com/shorts/abc"));
        assert!(!r.matches_url("https://www.youtube.com/watch?v=1"));
    }

    #[test]
    fn wildcard_all() {
        assert!(site_matches(&["*".into()], "https://anything.org"));
    }

    #[test]
    fn keywords_and_apps() {
        assert!(keyword_matches(&["Casino".into()], "https://a.com/best-casino"));
        assert!(!keyword_matches(&["".into()], "https://a.com"));
        assert!(app_matches(&["Steam.exe".into()], r"C:\Program Files\Steam\steam.EXE"));
        assert!(app_matches(&["discord".into()], "Discord.app"));
        assert!(!app_matches(&["steam".into()], "steamwebhelper.exe"));
    }
}
