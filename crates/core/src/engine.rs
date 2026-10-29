//! Computes what is blocked right now from the config and runtime state.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;

use crate::matcher::{app_matches, host_of, keyword_matches, site_matches, SiteRule};
use crate::model::{BlockList, Config, DailyStat, Limit, State, Usage};
use crate::schedule;

#[derive(Debug, Clone, Copy)]
pub struct Now {
    pub utc: DateTime<Utc>,
    pub local: NaiveDateTime,
}

impl Now {
    pub fn day(&self) -> String {
        self.local.format("%Y-%m-%d").to_string()
    }
    /// 0 = Monday … 6 = Sunday (local).
    pub fn weekday(&self) -> u8 {
        use chrono::Datelike;
        self.local.weekday().num_days_from_monday() as u8
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Session,
    Schedule,
    Limit,
    /// The list is set to "always on".
    Always,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ActiveBlock {
    pub block_list_id: String,
    pub block_list_name: String,
    pub reason: Reason,
    /// Session end (UTC) — absent for limits (end of day) and schedules (see `until_local`).
    pub until: Option<DateTime<Utc>>,
    pub until_local: Option<NaiveDateTime>,
    pub locked: bool,
}

/// A block list with a daily allowance that is not used up yet.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Metered {
    pub limit_id: String,
    pub block_list_id: String,
    pub remaining_seconds: u64,
    pub opens_left: Option<u32>,
}

/// One active list's rules, kept apart so its exceptions only lift its own blocks.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ListRules {
    pub sites: Vec<String>,
    pub apps: Vec<String>,
    pub keywords: Vec<String>,
    pub allow_sites: Vec<String>,
    pub allow_apps: Vec<String>,
}

impl ListRules {
    fn from(list: &BlockList) -> Self {
        ListRules {
            sites: list.sites.clone(),
            apps: list.apps.clone(),
            keywords: list.keywords.clone(),
            allow_sites: list.allow_sites.clone(),
            allow_apps: list.allow_apps.clone(),
        }
    }
    fn blocks_url(&self, url: &str) -> bool {
        (site_matches(&self.sites, url) || keyword_matches(&self.keywords, url))
            && !site_matches(&self.allow_sites, url)
    }
    fn blocks_app(&self, process_name: &str) -> bool {
        app_matches(&self.apps, process_name) && !app_matches(&self.allow_apps, process_name)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Policy {
    pub active: Vec<ActiveBlock>,
    /// Every blocked site rule (for the hosts file; see `host_exceptions`).
    pub blocked_sites: Vec<String>,
    pub blocked_apps: Vec<String>,
    pub keywords: Vec<String>,
    /// Site rules / words the browser extension may block instantly: those not
    /// touched by an exception. The rest is decided per page by the agent
    /// (`url_blocked`), which knows which list each exception belongs to.
    pub ext_sites: Vec<String>,
    pub ext_keywords: Vec<String>,
    /// Domains left out of the hosts file because of an exception, when no
    /// other active list blocks them.
    pub host_exceptions: Vec<String>,
    /// Per-list rules (the source of truth for pages and apps).
    pub rules: Vec<ListRules>,
    /// A page-level exception (e.g. `youtube.com/@channel`) took a whole
    /// domain out of the hosts file, so only the browser extension still
    /// blocks the rest of that site: browsers without it must not be used.
    pub needs_extension: bool,
    /// When true, only `allowed_sites` may be visited.
    pub allowlist_active: bool,
    pub allowed_sites: Vec<String>,
    pub metered: Vec<Metered>,
}

/// URLs that must never be blocked (browser internals, the agent itself).
fn is_internal(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    if [
        "chrome://",
        "edge://",
        "about:",
        "chrome-extension://",
        "moz-extension://",
        "file://",
        "brave://",
    ]
    .iter()
    .any(|p| u.starts_with(p))
    {
        return true;
    }
    matches!(host_of(&u).as_str(), "localhost" | "127.0.0.1" | "[::1]")
}

impl Policy {
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    pub fn url_blocked(&self, url: &str) -> bool {
        if is_internal(url) {
            return false;
        }
        if self.rules.iter().any(|r| r.blocks_url(url)) {
            return true;
        }
        self.allowlist_active && !site_matches(&self.allowed_sites, url)
    }

    pub fn app_blocked(&self, process_name: &str) -> bool {
        self.rules.iter().any(|r| r.blocks_app(process_name))
    }

    /// Whether a hosts-file entry should be left out because of an exception.
    pub fn host_excepted(&self, host: &str) -> bool {
        self.host_exceptions.iter().any(|e| {
            host == e
                || (host.len() > e.len()
                    && host.ends_with(e.as_str())
                    && host.as_bytes()[host.len() - e.len() - 1] == b'.')
        })
    }
}

fn usage_for<'a>(state: &'a State, day: &str, limit_id: &str) -> Option<&'a Usage> {
    state
        .usage
        .iter()
        .find(|u| u.day == day && u.limit_id == limit_id)
}

pub fn limit_exhausted(limit: &Limit, state: &State, day: &str, weekday: u8) -> bool {
    let minutes = limit.minutes_on(weekday);
    let Some(u) = usage_for(state, day, &limit.id) else {
        return minutes == 0;
    };
    u.seconds >= minutes as u64 * 60 || limit.max_opens.is_some_and(|m| u.opens >= m)
}

fn push_unique(v: &mut Vec<String>, items: &[String]) {
    for i in items {
        let t = i.trim();
        if !t.is_empty() && !v.iter().any(|x| x.eq_ignore_ascii_case(t)) {
            v.push(t.to_string());
        }
    }
}

pub fn effective_policy(config: &Config, state: &State, now: Now) -> Policy {
    let mut active: Vec<ActiveBlock> = Vec::new();
    let mut add = |list: &BlockList, reason, until, until_local, locked| {
        active.push(ActiveBlock {
            block_list_id: list.id.clone(),
            block_list_name: list.name.clone(),
            reason,
            until,
            until_local,
            locked,
        })
    };

    let protected = config.protection.level != crate::model::ProtectionLevel::Off;
    for list in config.block_lists.iter().filter(|b| b.always_on) {
        add(list, Reason::Always, None, None, protected);
    }
    for s in state.sessions.iter().filter(|s| s.is_active(now.utc)) {
        if let Some(list) = config.block_list(&s.block_list_id) {
            add(
                list,
                Reason::Session,
                Some(s.ends_at),
                None,
                s.is_locked(now.utc),
            );
        }
    }
    for s in config
        .schedules
        .iter()
        .filter(|s| schedule::is_active(s, now.local))
    {
        if let Some(list) = config.block_list(&s.block_list_id) {
            add(
                list,
                Reason::Schedule,
                None,
                schedule::current_window_end(s, now.local),
                s.locked,
            );
        }
    }
    let day = now.day();
    let mut metered = Vec::new();
    for l in config.limits.iter().filter(|l| l.enabled) {
        let Some(list) = config.block_list(&l.block_list_id) else {
            continue;
        };
        if limit_exhausted(l, state, &day, now.weekday()) {
            let midnight = (now.local.date() + chrono::Duration::days(1)).and_hms_opt(0, 0, 0);
            add(list, Reason::Limit, None, midnight, l.locked);
        } else {
            let u = usage_for(state, &day, &l.id);
            let used = u.map_or(0, |u| u.seconds);
            metered.push(Metered {
                limit_id: l.id.clone(),
                block_list_id: list.id.clone(),
                remaining_seconds: (l.minutes_on(now.weekday()) as u64 * 60).saturating_sub(used),
                opens_left: l
                    .max_opens
                    .map(|m| m.saturating_sub(u.map_or(0, |u| u.opens))),
            });
        }
    }

    let mut p = Policy {
        metered,
        ..Default::default()
    };
    let mut seen = Vec::new();
    for a in &active {
        if seen.contains(&a.block_list_id) {
            continue;
        }
        seen.push(a.block_list_id.clone());
        let list = config
            .block_list(&a.block_list_id)
            .expect("active list exists");
        if list.allowlist_mode {
            p.allowlist_active = true;
            push_unique(&mut p.allowed_sites, &list.sites);
            push_unique(&mut p.blocked_apps, &list.apps);
            p.rules.push(ListRules {
                apps: list.apps.clone(),
                allow_apps: list.allow_apps.clone(),
                ..Default::default()
            });
            continue;
        }
        push_unique(&mut p.blocked_sites, &list.sites);
        push_unique(&mut p.blocked_apps, &list.apps);
        push_unique(&mut p.keywords, &list.keywords);
        let excepted: Vec<SiteRule> = list
            .allow_sites
            .iter()
            .filter_map(|a| SiteRule::parse(a))
            .collect();
        let touched = |s: &String| {
            SiteRule::parse(s).is_some_and(|r| {
                excepted
                    .iter()
                    .any(|a| r.matches_host(&a.domain) || a.matches_host(&r.domain))
            })
        };
        let untouched: Vec<String> = list.sites.iter().filter(|s| !touched(s)).cloned().collect();
        push_unique(&mut p.ext_sites, &untouched);
        if excepted.is_empty() {
            push_unique(&mut p.ext_keywords, &list.keywords);
        }
        p.rules.push(ListRules::from(list));
    }
    // An exception keeps a domain out of the hosts file only if no other
    // active list (without that exception) blocks it there.
    for (i, r) in p.rules.iter().enumerate() {
        for a in r.allow_sites.iter().filter_map(|a| SiteRule::parse(a)) {
            if a.domain.is_empty() || !site_matches(&r.sites, &format!("https://{}/", a.domain)) {
                continue;
            }
            let probe = format!("https://{}/", a.domain);
            let elsewhere = p.rules.iter().enumerate().any(|(j, o)| {
                j != i && site_matches(&o.sites, &probe) && !site_matches(&o.allow_sites, &probe)
            });
            if elsewhere {
                continue;
            }
            if a.path_prefix.is_some() {
                p.needs_extension = true;
            }
            if !p.host_exceptions.contains(&a.domain) {
                p.host_exceptions.push(a.domain.clone());
            }
        }
    }
    p.active = active;
    p
}

fn list_matches_url(list: &BlockList, url: &str) -> bool {
    !list.allowlist_mode
        && (site_matches(&list.sites, url) || keyword_matches(&list.keywords, url))
        && !site_matches(&list.allow_sites, url)
}

/// Limits whose block list matches the URL.
pub fn limits_for_url<'a>(config: &'a Config, url: &str) -> Vec<&'a Limit> {
    config
        .limits
        .iter()
        .filter(|l| {
            l.enabled
                && config
                    .block_list(&l.block_list_id)
                    .is_some_and(|b| list_matches_url(b, url))
        })
        .collect()
}

pub fn limits_for_app<'a>(config: &'a Config, process_name: &str) -> Vec<&'a Limit> {
    config
        .limits
        .iter()
        .filter(|l| {
            l.enabled
                && config.block_list(&l.block_list_id).is_some_and(|b| {
                    app_matches(&b.apps, process_name) && !app_matches(&b.allow_apps, process_name)
                })
        })
        .collect()
}

/// Adds usage seconds (and optionally one "open") to the given limits for today.
pub fn record_usage(
    state: &mut State,
    limit_ids: &[String],
    day: &str,
    seconds: u64,
    opened: bool,
) {
    for id in limit_ids {
        let idx = match state
            .usage
            .iter()
            .position(|u| u.day == day && &u.limit_id == id)
        {
            Some(i) => i,
            None => {
                state.usage.push(Usage {
                    day: day.to_string(),
                    limit_id: id.clone(),
                    ..Default::default()
                });
                state.usage.len() - 1
            }
        };
        let u = &mut state.usage[idx];
        u.seconds += seconds;
        if opened {
            u.opens += 1;
        }
    }
}

/// Dashboard statistics: time per site/app key, and blocked attempts.
pub fn record_stat(state: &mut State, day: &str, key: &str, seconds: u64, blocked: bool) {
    let idx = match state
        .stats
        .iter()
        .position(|s| s.day == day && s.key == key)
    {
        Some(i) => i,
        None => {
            state.stats.push(DailyStat {
                day: day.to_string(),
                key: key.to_string(),
                ..Default::default()
            });
            state.stats.len() - 1
        }
    };
    state.stats[idx].seconds += seconds;
    if blocked {
        state.stats[idx].blocked_attempts += 1;
    }
}

/// Drops expired sessions, closed requests and usage/stats older than `keep_days`.
pub fn prune(state: &mut State, now: Now, keep_days: i64) {
    let cutoff = (now.local.date() - chrono::Duration::days(keep_days))
        .format("%Y-%m-%d")
        .to_string();
    state.sessions.retain(|s| s.ends_at > now.utc);
    state.usage.retain(|u| u.day >= cutoff);
    state.stats.retain(|s| s.day >= cutoff);
    let week_ago = now.utc - chrono::Duration::days(7);
    state
        .requests
        .retain(|r| r.is_open() || r.decided_at.is_none_or(|d| d > week_ago));
    if state
        .challenge
        .as_ref()
        .is_some_and(|c| c.expires_at <= now.utc)
    {
        state.challenge = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{LockKind, Schedule, Session};
    use chrono::{Duration, NaiveDate, TimeZone};

    fn now_at(h: u32) -> Now {
        let local = NaiveDate::from_ymd_opt(2026, 10, 12)
            .unwrap()
            .and_hms_opt(h, 0, 0)
            .unwrap();
        Now {
            utc: Utc.from_utc_datetime(&local),
            local,
        }
    }

    fn config() -> Config {
        Config {
            block_lists: vec![
                BlockList {
                    id: "social".into(),
                    name: "Social".into(),
                    sites: vec!["facebook.com".into(), "youtube.com/shorts".into()],
                    apps: vec!["discord.exe".into()],
                    keywords: vec!["tiktok".into()],
                    allowlist_mode: false,
                    always_on: false,
                    preset: String::new(),
                    ..Default::default()
                },
                BlockList {
                    id: "work".into(),
                    name: "Work only".into(),
                    sites: vec!["github.com".into()],
                    allowlist_mode: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn nothing_active() {
        let p = effective_policy(&config(), &State::default(), now_at(10));
        assert!(p.is_empty());
        assert!(!p.url_blocked("https://facebook.com"));
    }

    #[test]
    fn session_blocks() {
        let now = now_at(10);
        let state = State {
            sessions: vec![Session {
                id: "s".into(),
                block_list_id: "social".into(),
                started_at: now.utc,
                ends_at: now.utc + Duration::hours(1),
                lock: LockKind::Timer,
            }],
            ..Default::default()
        };
        let p = effective_policy(&config(), &state, now);
        assert!(p.active[0].locked);
        assert!(p.url_blocked("https://www.facebook.com/"));
        assert!(p.url_blocked("https://www.youtube.com/shorts/x"));
        assert!(!p.url_blocked("https://www.youtube.com/watch?v=1"));
        assert!(p.url_blocked("https://example.com/tiktok-video"));
        assert!(p.app_blocked("Discord.exe"));
        assert!(!p.url_blocked("chrome://settings"));
    }

    #[test]
    fn allowlist_schedule() {
        let mut c = config();
        c.schedules.push(Schedule {
            id: "w".into(),
            block_list_id: "work".into(),
            days: vec![0],
            start_minute: 9 * 60,
            end_minute: 17 * 60,
            enabled: true,
            locked: false,
        });
        let p = effective_policy(&c, &State::default(), now_at(10));
        assert!(p.allowlist_active);
        assert!(!p.url_blocked("https://github.com/x"));
        assert!(p.url_blocked("https://news.ycombinator.com"));
        assert!(!p.url_blocked("http://localhost:17345/"));
    }

    #[test]
    fn limits_meter_then_block() {
        let mut c = config();
        c.limits.push(Limit {
            id: "l".into(),
            block_list_id: "social".into(),
            daily_minutes: 30,
            max_opens: Some(3),
            enabled: true,
            locked: false,
            ..Default::default()
        });
        let now = now_at(10);
        let mut state = State::default();
        let p = effective_policy(&c, &state, now);
        assert!(p.is_empty());
        assert_eq!(p.metered[0].remaining_seconds, 1800);

        let ids: Vec<String> = limits_for_url(&c, "https://facebook.com")
            .iter()
            .map(|l| l.id.clone())
            .collect();
        assert_eq!(ids, vec!["l".to_string()]);
        record_usage(&mut state, &ids, &now.day(), 1200, true);
        assert_eq!(
            effective_policy(&c, &state, now).metered[0].remaining_seconds,
            600
        );
        record_usage(&mut state, &ids, &now.day(), 600, false);
        let p = effective_policy(&c, &state, now);
        assert_eq!(p.active[0].reason, Reason::Limit);
        assert!(p.url_blocked("https://facebook.com"));

        // A new day resets the allowance.
        let mut tomorrow = now;
        tomorrow.local += Duration::days(1);
        assert!(effective_policy(&c, &state, tomorrow).is_empty());
    }

    #[test]
    fn weekend_limit_gives_more_time() {
        let mut c = config();
        c.limits.push(Limit {
            id: "l".into(),
            block_list_id: "social".into(),
            daily_minutes: 30,
            day_minutes: vec![30, 30, 30, 30, 30, 120, 0],
            enabled: true,
            ..Default::default()
        });
        let monday = now_at(10);
        assert_eq!(monday.weekday(), 0);
        let p = effective_policy(&c, &State::default(), monday);
        assert_eq!(p.metered[0].remaining_seconds, 30 * 60);
        let mut saturday = monday;
        saturday.local += Duration::days(5);
        let p = effective_policy(&c, &State::default(), saturday);
        assert_eq!(p.metered[0].remaining_seconds, 120 * 60);
        // 0 minutes on Sunday = blocked all day.
        let mut sunday = monday;
        sunday.local += Duration::days(6);
        let p = effective_policy(&c, &State::default(), sunday);
        assert_eq!(p.active[0].reason, Reason::Limit);
    }

    #[test]
    fn opens_limit() {
        let mut c = config();
        c.limits.push(Limit {
            id: "l".into(),
            block_list_id: "social".into(),
            daily_minutes: 600,
            max_opens: Some(2),
            enabled: true,
            locked: false,
            ..Default::default()
        });
        let now = now_at(10);
        let mut state = State::default();
        record_usage(&mut state, &["l".into()], &now.day(), 1, true);
        record_usage(&mut state, &["l".into()], &now.day(), 1, true);
        assert!(!effective_policy(&c, &state, now).is_empty());
    }

    #[test]
    fn page_exceptions_need_the_extension() {
        let mut c = config();
        c.block_lists = vec![always(
            "v",
            &["youtube.com"],
            &[],
            &["youtube.com/@khanacademy"],
        )];
        assert!(effective_policy(&c, &State::default(), now_at(10)).needs_extension);
        // A subdomain exception stays exact in the hosts file: no extension needed.
        c.block_lists = vec![always("g", &["google.com"], &[], &["mail.google.com"])];
        assert!(!effective_policy(&c, &State::default(), now_at(10)).needs_extension);
        // Another list still blocks the whole domain: it stays in the hosts file.
        c.block_lists = vec![
            always("v", &["youtube.com"], &[], &["youtube.com/@khanacademy"]),
            always("w", &["youtube.com"], &[], &[]),
        ];
        let p = effective_policy(&c, &State::default(), now_at(10));
        assert!(!p.needs_extension && p.host_exceptions.is_empty());
    }

    fn always(id: &str, sites: &[&str], keywords: &[&str], allow: &[&str]) -> BlockList {
        let v = |x: &[&str]| x.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        BlockList {
            id: id.into(),
            name: id.into(),
            sites: v(sites),
            keywords: v(keywords),
            allow_sites: v(allow),
            always_on: true,
            ..Default::default()
        }
    }

    #[test]
    fn exceptions_lift_only_their_own_list() {
        let now = now_at(10);
        let st = State::default();
        // Video list blocks YouTube but allows one educational channel.
        let mut c = Config::default();
        c.block_lists.push(always(
            "video",
            &["youtube.com"],
            &[],
            &["youtube.com/@khanacademy"],
        ));
        let p = effective_policy(&c, &st, now);
        assert!(p.url_blocked("https://www.youtube.com/watch?v=1"));
        assert!(!p.url_blocked("https://www.youtube.com/@khanacademy/videos"));
        // The extension mustn't block YouTube instantly (it would hit the
        // exception); the agent decides those pages. Path exceptions keep the
        // domain out of the hosts file too.
        assert!(p.ext_sites.is_empty());
        assert!(p.host_excepted("youtube.com") && p.host_excepted("www.youtube.com"));

        // Another list that blocks YouTube without that exception still wins.
        c.block_lists
            .push(always("social", &["youtube.com", "facebook.com"], &[], &[]));
        let p = effective_policy(&c, &st, now);
        assert!(p.url_blocked("https://www.youtube.com/@khanacademy/videos"));
        assert!(!p.host_excepted("youtube.com"));
        assert!(p.ext_sites.contains(&"facebook.com".to_string()));
    }

    #[test]
    fn exceptions_for_subdomains_words_and_apps() {
        let now = now_at(10);
        let st = State::default();
        let mut c = Config::default();
        // Block Google except Gmail; block "sex-" words except a university site.
        c.block_lists.push(always(
            "g",
            &["google.com"],
            &["sex-"],
            &["mail.google.com", "essex-uni.ac.uk"],
        ));
        let mut games = always("games", &[], &[], &[]);
        games.apps = vec!["steam.exe".into(), "minecraft.exe".into()];
        games.allow_apps = vec!["minecraft.exe".into()];
        c.block_lists.push(games);
        let p = effective_policy(&c, &st, now);
        assert!(p.url_blocked("https://google.com/search?q=x"));
        assert!(!p.url_blocked("https://mail.google.com/mail/u/0"));
        assert!(p.host_excepted("mail.google.com") && !p.host_excepted("google.com"));
        assert!(p.url_blocked("https://some-sex-site.example/"));
        assert!(!p.url_blocked("https://www.essex-uni.ac.uk/sex-education"));
        // A list with exceptions doesn't hand its words to the extension.
        assert!(p.ext_keywords.is_empty());
        assert!(p.app_blocked("Steam.exe") && !p.app_blocked("minecraft.exe"));
    }
}
