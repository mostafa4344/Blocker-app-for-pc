//! Lock rules: while something is locked, settings can only become stricter.
//! Sessions can only be ended early through their lock's unlock method.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::model::{BlockList, Config, LockKind, Session, State};
use crate::protection::is_at_least_as_strict;
use crate::schedule::covers;

/// Maximum lock length while practice mode is on (first days after install).
pub const PRACTICE_MAX_LOCK_MINUTES: i64 = 120;
pub const MAX_SESSION_MINUTES: i64 = 7 * 24 * 60;

#[derive(Debug, Clone, PartialEq, Serialize, thiserror::Error)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PolicyError {
    #[error("block list '{name}' is locked and cannot be loosened")]
    ListLocked { name: String },
    #[error("schedule is locked and can only be extended")]
    ScheduleLocked { id: String },
    #[error("limit is locked and can only be tightened")]
    LimitLocked { id: String },
    #[error("lowering protection needs a protection request")]
    ProtectionRequiresRequest,
    #[error("browser protection '{what}' cannot be turned off while protected")]
    EnforcementLocked { what: String },
    #[error("session is locked")]
    SessionLocked { id: String },
    #[error("not found: {what}")]
    NotFound { what: String },
    #[error("invalid: {what}")]
    Invalid { what: String },
}

/// True when protection is on: always-on lists and enabled schedules/limits
/// then behave as locked (they can only be made stricter). Loosening them needs
/// lowering protection first, which goes through a protection request.
pub fn guarded(config: &Config) -> bool {
    config.protection.level != crate::model::ProtectionLevel::Off
}

/// Block list ids that are currently locked.
pub fn locked_list_ids(config: &Config, state: &State, now: DateTime<Utc>) -> Vec<String> {
    let guarded = guarded(config);
    let mut ids: Vec<String> = Vec::new();
    let mut push = |id: &String| {
        if !ids.contains(id) {
            ids.push(id.clone())
        }
    };
    state
        .sessions
        .iter()
        .filter(|s| s.is_locked(now))
        .for_each(|s| push(&s.block_list_id));
    config
        .schedules
        .iter()
        .filter(|s| s.enabled && (s.locked || guarded))
        .for_each(|s| push(&s.block_list_id));
    config
        .limits
        .iter()
        .filter(|l| l.enabled && (l.locked || guarded))
        .for_each(|l| push(&l.block_list_id));
    config
        .block_lists
        .iter()
        .filter(|b| b.always_on && guarded)
        .for_each(|b| push(&b.id));
    ids
}

fn norm(v: &[String]) -> Vec<String> {
    v.iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn superset(new: &[String], old: &[String]) -> bool {
    let n = norm(new);
    norm(old).iter().all(|o| n.contains(o))
}

fn list_is_stricter(new: &BlockList, old: &BlockList) -> bool {
    if new.allowlist_mode != old.allowlist_mode || (old.always_on && !new.always_on) {
        return false;
    }
    // A ready-made list keeps its tag: without it the list would silently stop
    // receiving the new sites we add to it from the server.
    if !old.preset.is_empty() && new.preset != old.preset {
        return false;
    }
    let sites_ok = if old.allowlist_mode {
        superset(&old.sites, &new.sites)
    } else {
        superset(&new.sites, &old.sites)
    };
    // Exceptions only shrink: adding one would open part of a locked list.
    sites_ok
        && superset(&new.apps, &old.apps)
        && superset(&new.keywords, &old.keywords)
        && superset(&old.allow_sites, &new.allow_sites)
        && superset(&old.allow_apps, &new.allow_apps)
}

/// Cleans the requested exceptions: drops blanks and the ones the list
/// already has; refuses unknown lists, invalid sites and "*".
pub fn new_exceptions(
    config: &Config,
    list_id: &str,
    sites: &[String],
    apps: &[String],
) -> Result<(Vec<String>, Vec<String>), PolicyError> {
    let list = config
        .block_list(list_id)
        .ok_or_else(|| PolicyError::NotFound {
            what: "block_list".into(),
        })?;
    let fresh = |items: &[String], have: &[String]| {
        let mut out: Vec<String> = Vec::new();
        for i in items.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !have
                .iter()
                .chain(out.iter())
                .any(|h| h.eq_ignore_ascii_case(i))
            {
                out.push(i.to_string());
            }
        }
        out
    };
    let sites = fresh(sites, &list.allow_sites);
    let apps = fresh(apps, &list.allow_apps);
    if sites
        .iter()
        .any(|a| crate::matcher::SiteRule::parse(a).is_none_or(|r| r.domain.is_empty()))
    {
        return Err(PolicyError::Invalid {
            what: "block_list.allow_sites".into(),
        });
    }
    if sites.is_empty() && apps.is_empty() {
        return Err(PolicyError::Invalid {
            what: "exceptions.empty".into(),
        });
    }
    Ok((sites, apps))
}

/// Adds exceptions to a list (used when an approved request is applied).
pub fn add_exceptions(
    config: &mut Config,
    list_id: &str,
    sites: &[String],
    apps: &[String],
) -> Result<(), PolicyError> {
    let (sites, apps) = match new_exceptions(config, list_id, sites, apps) {
        Ok(v) => v,
        // Already there (e.g. added again meanwhile): nothing to do.
        Err(PolicyError::Invalid { what }) if what == "exceptions.empty" => return Ok(()),
        Err(e) => return Err(e),
    };
    if let Some(l) = config.block_lists.iter_mut().find(|b| b.id == list_id) {
        l.allow_sites.extend(sites);
        l.allow_apps.extend(apps);
    }
    validate_shape(config)
}

/// Validates replacing `old` with `new` given what is currently locked.
pub fn validate_update(
    old: &Config,
    new: &Config,
    state: &State,
    now: DateTime<Utc>,
) -> Result<(), PolicyError> {
    for id in locked_list_ids(old, state, now) {
        let Some(o) = old.block_list(&id) else {
            continue;
        };
        match new.block_list(&id) {
            Some(n) if list_is_stricter(n, o) => {}
            _ => {
                return Err(PolicyError::ListLocked {
                    name: o.name.clone(),
                })
            }
        }
    }
    let guarded = guarded(old);
    for o in old
        .schedules
        .iter()
        .filter(|s| s.enabled && (s.locked || guarded))
    {
        let ok = new
            .schedules
            .iter()
            .find(|n| n.id == o.id)
            .is_some_and(|n| {
                n.enabled
                    && (n.locked || !o.locked)
                    && n.block_list_id == o.block_list_id
                    && covers(n, o)
            });
        if !ok {
            return Err(PolicyError::ScheduleLocked { id: o.id.clone() });
        }
    }
    for o in old
        .limits
        .iter()
        .filter(|l| l.enabled && (l.locked || guarded))
    {
        let ok = new.limits.iter().find(|n| n.id == o.id).is_some_and(|n| {
            n.enabled
                && (n.locked || !o.locked)
                && n.block_list_id == o.block_list_id
                && (0..7).all(|d| n.minutes_on(d) <= o.minutes_on(d))
                && match (o.max_opens, n.max_opens) {
                    (Some(a), Some(b)) => b <= a,
                    (Some(_), None) => false,
                    _ => true,
                }
        });
        if !ok {
            return Err(PolicyError::LimitLocked { id: o.id.clone() });
        }
    }
    if !is_at_least_as_strict(&new.protection, &old.protection) {
        return Err(PolicyError::ProtectionRequiresRequest);
    }
    // Browser-enforcement switches can only tighten (be turned on) while the
    // user is protected: with protection above Off, or anything locked. This
    // closes the "just untick 'block unsupported browsers' / 'lock extension'"
    // bypass that a determined user would otherwise reach for.
    let protected = old.protection.level != crate::model::ProtectionLevel::Off
        || !locked_list_ids(old, state, now).is_empty();
    if protected {
        let turned_off = |o: bool, n: bool| o && !n;
        let s_old = &old.settings;
        let s_new = &new.settings;
        for (what, o, n) in [
            (
                "require_browser_extension",
                s_old.require_browser_extension,
                s_new.require_browser_extension,
            ),
            (
                "block_unsupported_browsers",
                s_old.block_unsupported_browsers,
                s_new.block_unsupported_browsers,
            ),
            (
                "managed_extension",
                s_old.managed_extension,
                s_new.managed_extension,
            ),
            (
                "force_safe_search",
                s_old.force_safe_search,
                s_new.force_safe_search,
            ),
            (
                "block_private_windows",
                s_old.block_private_windows,
                s_new.block_private_windows,
            ),
        ] {
            if turned_off(o, n) {
                return Err(PolicyError::EnforcementLocked { what: what.into() });
            }
        }
        // Partial blocking (hide Shorts, Reels, feeds...): filters can be added,
        // never removed, or unticking "Hide YouTube Shorts" would undo it.
        if let Some(f) = s_old
            .distraction_filters
            .iter()
            .find(|f| !s_new.distraction_filters.contains(f))
        {
            return Err(PolicyError::EnforcementLocked {
                what: format!("distraction_filters:{f}"),
            });
        }
        // The DNS filter can be switched on (or to another provider), not off.
        if s_old.dns_filter != "off" && s_new.dns_filter == "off" {
            return Err(PolicyError::EnforcementLocked {
                what: "dns_filter".into(),
            });
        }
    }
    validate_shape(new)
}

/// Structural sanity checks independent of locks.
pub fn validate_shape(c: &Config) -> Result<(), PolicyError> {
    let invalid = |w: &str| {
        Err(PolicyError::Invalid {
            what: w.to_string(),
        })
    };
    for (i, b) in c.block_lists.iter().enumerate() {
        if b.id.trim().is_empty() || c.block_lists[..i].iter().any(|x| x.id == b.id) {
            return invalid("block_list.id");
        }
        if b.name.trim().is_empty() {
            return invalid("block_list.name");
        }
        // An exception must be a real site or page: "*" would allow everything.
        if b.allow_sites
            .iter()
            .any(|a| crate::matcher::SiteRule::parse(a).is_none_or(|r| r.domain.is_empty()))
        {
            return invalid("block_list.allow_sites");
        }
    }
    for s in &c.schedules {
        if c.block_list(&s.block_list_id).is_none() {
            return invalid("schedule.block_list_id");
        }
        if s.start_minute >= 24 * 60 || s.end_minute >= 24 * 60 || s.days.iter().any(|d| *d > 6) {
            return invalid("schedule.time");
        }
    }
    for l in &c.limits {
        if c.block_list(&l.block_list_id).is_none() {
            return invalid("limit.block_list_id");
        }
        if l.daily_minutes > 24 * 60 {
            return invalid("limit.daily_minutes");
        }
        if !(l.day_minutes.is_empty() || l.day_minutes.len() == 7)
            || l.day_minutes.iter().any(|m| *m > 24 * 60)
        {
            return invalid("limit.day_minutes");
        }
    }
    if let Some(p) = &c.protection.partner {
        if !p.email.contains('@') {
            return invalid("protection.partner.email");
        }
    }
    if c.protection.level == crate::model::ProtectionLevel::Partner
        && c.protection.partner.is_none()
    {
        return invalid("protection.partner");
    }
    Ok(())
}

pub fn start_session(
    config: &Config,
    block_list_id: &str,
    minutes: i64,
    lock: LockKind,
    now: DateTime<Utc>,
) -> Result<Session, PolicyError> {
    if !(1..=MAX_SESSION_MINUTES).contains(&minutes) {
        return Err(PolicyError::Invalid {
            what: "minutes".into(),
        });
    }
    start_session_between(
        config,
        block_list_id,
        now,
        now + Duration::minutes(minutes),
        lock,
        now,
    )
}

/// How far ahead a session may be planned.
pub const MAX_PLAN_AHEAD_DAYS: i64 = 7;

/// A session from `start` to `end` ("block from 4 pm until 7 am tomorrow").
/// `start` may be now or later (up to a week ahead); a start in the past
/// counts as now.
pub fn start_session_between(
    config: &Config,
    block_list_id: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    lock: LockKind,
    now: DateTime<Utc>,
) -> Result<Session, PolicyError> {
    let invalid = |w: &str| {
        Err(PolicyError::Invalid {
            what: w.to_string(),
        })
    };
    if config.block_list(block_list_id).is_none() {
        return Err(PolicyError::NotFound {
            what: "block_list".into(),
        });
    }
    let start = start.max(now);
    if start > now + Duration::days(MAX_PLAN_AHEAD_DAYS) {
        return invalid("starts_at");
    }
    let minutes = (end - start).num_minutes();
    if !(1..=MAX_SESSION_MINUTES).contains(&minutes) {
        return invalid("ends_at");
    }
    match lock {
        LockKind::RandomText { length } if !(10..=2000).contains(&length) => {
            return invalid("lock.length")
        }
        LockKind::Partner if config.protection.partner.is_none() => {
            return invalid("protection.partner")
        }
        LockKind::Window {
            start_minute,
            end_minute,
        } if start_minute >= 24 * 60 || end_minute >= 24 * 60 || start_minute == end_minute => {
            return invalid("lock.window")
        }
        _ => {}
    }
    let practice = config.settings.practice_mode_until.is_some_and(|t| now < t);
    let minutes = if practice && lock != LockKind::None {
        minutes.min(PRACTICE_MAX_LOCK_MINUTES)
    } else {
        minutes
    };
    Ok(Session {
        id: crate::new_id(),
        block_list_id: block_list_id.to_string(),
        started_at: start,
        ends_at: start + Duration::minutes(minutes),
        lock,
    })
}

/// True when `minute` (local minutes since midnight) is inside the daily
/// window `start..end`, which may cross midnight.
pub fn in_daily_window(minute: u32, start: u16, end: u16) -> bool {
    let (s, e) = (start as u32, end as u32);
    if s < e {
        (s..e).contains(&minute)
    } else {
        minute >= s || minute < e
    }
}

/// What the user has done to justify ending a session early.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockProof {
    Nothing,
    ChallengeSolved,
    /// The computer was restarted after the session started.
    Restarted,
    PartnerApproved,
}

/// Whether a session may be ended now. `local_minute` is the local time of
/// day (minutes since midnight) for window locks. A planned session that
/// hasn't started yet follows the same rule: the commitment holds once made.
pub fn can_end_session(
    session: &Session,
    proof: UnlockProof,
    now: DateTime<Utc>,
    local_minute: u32,
) -> Result<(), PolicyError> {
    if now >= session.ends_at {
        return Ok(());
    }
    let ok = match session.lock {
        LockKind::None => true,
        LockKind::Timer => false,
        LockKind::RandomText { .. } => proof == UnlockProof::ChallengeSolved,
        LockKind::Restart => proof == UnlockProof::Restarted,
        LockKind::Partner => proof == UnlockProof::PartnerApproved,
        LockKind::Window {
            start_minute,
            end_minute,
        } => in_daily_window(local_minute, start_minute, end_minute),
    };
    if ok {
        Ok(())
    } else {
        Err(PolicyError::SessionLocked {
            id: session.id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Limit, Protection, ProtectionLevel, Schedule};

    fn base() -> Config {
        Config {
            block_lists: vec![BlockList {
                id: "a".into(),
                name: "A".into(),
                sites: vec!["x.com".into()],
                ..Default::default()
            }],
            schedules: vec![Schedule {
                id: "s".into(),
                block_list_id: "a".into(),
                days: vec![0, 1],
                start_minute: 540,
                end_minute: 1020,
                enabled: true,
                locked: true,
            }],
            limits: vec![Limit {
                id: "l".into(),
                block_list_id: "a".into(),
                daily_minutes: 30,
                max_opens: Some(5),
                enabled: true,
                locked: true,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn can_add_but_not_remove_sites() {
        let old = base();
        let now = Utc::now();
        let mut more = old.clone();
        more.block_lists[0].sites.push("y.com".into());
        assert!(validate_update(&old, &more, &State::default(), now).is_ok());
        let mut less = old.clone();
        less.block_lists[0].sites.clear();
        assert!(matches!(
            validate_update(&old, &less, &State::default(), now),
            Err(PolicyError::ListLocked { .. })
        ));
        let mut deleted = old.clone();
        deleted.block_lists.clear();
        deleted.schedules.clear();
        deleted.limits.clear();
        assert!(validate_update(&old, &deleted, &State::default(), now).is_err());
    }

    #[test]
    fn exceptions_only_through_a_request() {
        let old = base();
        let now = Utc::now();
        let v = |x: &[&str]| x.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        // Adding directly to a locked list is refused...
        let mut direct = old.clone();
        direct.block_lists[0].allow_sites = v(&["mail.google.com"]);
        assert!(validate_update(&old, &direct, &State::default(), now).is_err());
        // ...the request path cleans and checks the entries...
        assert!(new_exceptions(&old, "a", &v(&["*"]), &[]).is_err());
        assert!(new_exceptions(&old, "zzz", &v(&["x.com"]), &[]).is_err());
        assert!(new_exceptions(&old, "a", &v(&[" ", ""]), &[]).is_err());
        let (sites, apps) = new_exceptions(
            &old,
            "a",
            &v(&["mail.google.com", "MAIL.google.com"]),
            &v(&["game.exe"]),
        )
        .unwrap();
        assert_eq!((sites.len(), apps.len()), (1, 1));
        // ...and an approved request adds them once.
        let mut next = old.clone();
        add_exceptions(&mut next, "a", &sites, &apps).unwrap();
        add_exceptions(&mut next, "a", &sites, &apps).unwrap();
        assert_eq!(next.block_lists[0].allow_sites, v(&["mail.google.com"]));
        assert_eq!(next.block_lists[0].allow_apps, v(&["game.exe"]));
    }

    #[test]
    fn locked_schedule_and_limit() {
        let old = base();
        let now = Utc::now();
        let mut shorter = old.clone();
        shorter.schedules[0].end_minute = 600;
        assert!(matches!(
            validate_update(&old, &shorter, &State::default(), now),
            Err(PolicyError::ScheduleLocked { .. })
        ));
        let mut looser = old.clone();
        looser.limits[0].daily_minutes = 60;
        assert!(matches!(
            validate_update(&old, &looser, &State::default(), now),
            Err(PolicyError::LimitLocked { .. })
        ));
        // More time on one weekday is looser too.
        let mut weekend = old.clone();
        weekend.limits[0].day_minutes = vec![30, 30, 30, 30, 30, 90, 30];
        assert!(matches!(
            validate_update(&old, &weekend, &State::default(), now),
            Err(PolicyError::LimitLocked { .. })
        ));
        let mut fewer = old.clone();
        fewer.limits[0].day_minutes = vec![30, 30, 30, 30, 30, 10, 0];
        assert!(validate_update(&old, &fewer, &State::default(), now).is_ok());
        let mut bad = old.clone();
        bad.limits[0].day_minutes = vec![10, 10];
        assert!(validate_update(&old, &bad, &State::default(), now).is_err());
        let mut tighter = old.clone();
        tighter.limits[0].daily_minutes = 10;
        tighter.limits[0].max_opens = Some(2);
        tighter.schedules[0].days.push(2);
        assert!(validate_update(&old, &tighter, &State::default(), now).is_ok());
    }

    #[test]
    fn unlocked_config_is_free() {
        let mut old = base();
        old.schedules[0].locked = false;
        old.limits[0].locked = false;
        let mut new = old.clone();
        new.block_lists[0].sites.clear();
        new.schedules.clear();
        new.limits.clear();
        assert!(validate_update(&old, &new, &State::default(), Utc::now()).is_ok());
    }

    #[test]
    fn protection_cannot_be_lowered_directly() {
        let old = Config {
            protection: Protection {
                level: ProtectionLevel::Committed,
                ..Default::default()
            },
            ..Default::default()
        };
        let new = Config::default();
        assert_eq!(
            validate_update(&old, &new, &State::default(), Utc::now()),
            Err(PolicyError::ProtectionRequiresRequest)
        );
    }

    #[test]
    fn session_lock_kinds() {
        let now = Utc::now();
        let c = base();
        let s = start_session(&c, "a", 60, LockKind::Timer, now).unwrap();
        assert!(can_end_session(&s, UnlockProof::ChallengeSolved, now, 600).is_err());
        assert!(
            can_end_session(&s, UnlockProof::Nothing, now + Duration::minutes(61), 600).is_ok()
        );
        let r = start_session(&c, "a", 60, LockKind::RandomText { length: 100 }, now).unwrap();
        assert!(can_end_session(&r, UnlockProof::ChallengeSolved, now, 600).is_ok());
        assert!(can_end_session(&r, UnlockProof::Restarted, now, 600).is_err());
        assert!(start_session(&c, "a", 60, LockKind::Partner, now).is_err());
    }

    #[test]
    fn enforcement_settings_tighten_only_while_protected() {
        let now = Utc::now();
        // Protected because a schedule is locked (base() has one).
        let old = {
            let mut c = base();
            c.settings.block_unsupported_browsers = true;
            c.settings.managed_extension = true;
            c
        };
        // Turning a browser switch off is rejected.
        let mut off = old.clone();
        off.settings.block_unsupported_browsers = false;
        assert!(matches!(
            validate_update(&old, &off, &State::default(), now),
            Err(PolicyError::EnforcementLocked { .. })
        ));
        // Turning one on (tightening) is fine.
        let mut on = old.clone();
        on.settings.require_browser_extension = true;
        assert!(validate_update(&old, &on, &State::default(), now).is_ok());

        // Partial-blocking filters can be added but not removed.
        let mut filters = old.clone();
        filters.settings.distraction_filters = vec!["youtube_shorts".into()];
        let mut more = filters.clone();
        more.settings
            .distraction_filters
            .push("facebook_reels".into());
        assert!(validate_update(&filters, &more, &State::default(), now).is_ok());
        let mut fewer = filters.clone();
        fewer.settings.distraction_filters.clear();
        assert!(matches!(
            validate_update(&filters, &fewer, &State::default(), now),
            Err(PolicyError::EnforcementLocked { .. })
        ));
        // The DNS filter can't be switched back off.
        let mut dns = old.clone();
        dns.settings.dns_filter = "family".into();
        let mut dns_off = dns.clone();
        dns_off.settings.dns_filter = "off".into();
        assert!(validate_update(&dns, &dns_off, &State::default(), now).is_err());
        assert!(validate_update(&old, &dns, &State::default(), now).is_ok());

        // With nothing locked and protection Off, switches are free to toggle.
        let mut unprotected = old.clone();
        unprotected.schedules[0].locked = false;
        unprotected.limits[0].locked = false;
        let mut free = unprotected.clone();
        free.settings.managed_extension = false;
        free.settings.block_unsupported_browsers = false;
        assert!(validate_update(&unprotected, &free, &State::default(), now).is_ok());
    }

    #[test]
    fn planned_sessions_and_end_times() {
        let now = Utc::now();
        let c = base();
        // "From 4 hours from now until 10 hours from now".
        let s = start_session_between(
            &c,
            "a",
            now + Duration::hours(4),
            now + Duration::hours(10),
            LockKind::Timer,
            now,
        )
        .unwrap();
        assert_eq!(s.started_at, now + Duration::hours(4));
        assert!(!s.is_active(now));
        // A planned lock is binding at once: its list is guarded and it can't be cancelled.
        assert!(s.is_locked(now));
        let mut st = State::default();
        st.sessions.push(s.clone());
        assert!(locked_list_ids(&c, &st, now).contains(&"a".to_string()));
        assert!(can_end_session(&s, UnlockProof::Nothing, now, 600).is_err());
        assert!(can_end_session(&s, UnlockProof::Nothing, now + Duration::hours(10), 600).is_ok());
        // A start in the past counts as now; the end must be after the start.
        let past = start_session_between(
            &c,
            "a",
            now - Duration::hours(1),
            now + Duration::hours(1),
            LockKind::None,
            now,
        )
        .unwrap();
        assert_eq!(past.started_at, now);
        assert!(start_session_between(&c, "a", now, now, LockKind::None, now).is_err());
        assert!(start_session_between(
            &c,
            "a",
            now + Duration::days(8),
            now + Duration::days(9),
            LockKind::None,
            now
        )
        .is_err());
        assert!(
            start_session_between(&c, "a", now, now + Duration::days(8), LockKind::None, now)
                .is_err()
        );
    }

    #[test]
    fn window_lock_opens_only_in_its_window() {
        let now = Utc::now();
        let c = base();
        // Can only be stopped between 20:00 and 21:00.
        let w = LockKind::Window {
            start_minute: 20 * 60,
            end_minute: 21 * 60,
        };
        let s = start_session(&c, "a", 600, w, now).unwrap();
        assert!(can_end_session(&s, UnlockProof::Nothing, now, 19 * 60 + 59).is_err());
        assert!(can_end_session(&s, UnlockProof::Nothing, now, 20 * 60).is_ok());
        assert!(can_end_session(&s, UnlockProof::Nothing, now, 21 * 60).is_err());
        // Windows may cross midnight (23:00-01:00).
        assert!(in_daily_window(30, 23 * 60, 60));
        assert!(in_daily_window(23 * 60 + 5, 23 * 60, 60));
        assert!(!in_daily_window(12 * 60, 23 * 60, 60));
        // An empty or impossible window is refused.
        let empty = LockKind::Window {
            start_minute: 600,
            end_minute: 600,
        };
        assert!(start_session(&c, "a", 60, empty, now).is_err());
        let bad = LockKind::Window {
            start_minute: 0,
            end_minute: 1440,
        };
        assert!(start_session(&c, "a", 60, bad, now).is_err());
    }

    #[test]
    fn practice_mode_caps_locks() {
        let now = Utc::now();
        let mut c = base();
        c.settings.practice_mode_until = Some(now + Duration::days(7));
        let s = start_session(&c, "a", 600, LockKind::Timer, now).unwrap();
        assert_eq!(
            s.ends_at - s.started_at,
            Duration::minutes(PRACTICE_MAX_LOCK_MINUTES)
        );
        let free = start_session(&c, "a", 600, LockKind::None, now).unwrap();
        assert_eq!(free.ends_at - free.started_at, Duration::minutes(600));
    }
    #[test]
    fn protection_guards_always_on_lists_and_schedules() {
        let now = Utc::now();
        let st = State::default();
        let mut old = Config::default();
        old.block_lists.push(BlockList {
            id: "adult".into(),
            name: "Adult".into(),
            sites: vec!["bad.com".into()],
            always_on: true,
            ..Default::default()
        });
        old.schedules.push(Schedule {
            id: "s".into(),
            block_list_id: "adult".into(),
            days: vec![0],
            start_minute: 0,
            end_minute: 600,
            enabled: true,
            locked: false,
        });
        // Protection off: everything may be loosened freely.
        let mut loose = old.clone();
        loose.block_lists[0].always_on = false;
        loose.block_lists[0].sites.clear();
        loose.schedules[0].enabled = false;
        assert!(validate_update(&old, &loose, &st, now).is_ok());

        // Protection on: the always-on list and the schedule are guarded.
        old.protection = Protection {
            level: ProtectionLevel::Committed,
            ..Default::default()
        };
        loose.protection = old.protection.clone();
        assert!(locked_list_ids(&old, &st, now).contains(&"adult".to_string()));
        let mut off = old.clone();
        off.block_lists[0].always_on = false;
        assert!(validate_update(&old, &off, &st, now).is_err());
        let mut fewer = old.clone();
        fewer.block_lists[0].sites.clear();
        assert!(validate_update(&old, &fewer, &st, now).is_err());
        let mut deleted = old.clone();
        deleted.block_lists.clear();
        deleted.schedules.clear();
        assert!(validate_update(&old, &deleted, &st, now).is_err());
        let mut sched_off = old.clone();
        sched_off.schedules[0].enabled = false;
        assert!(validate_update(&old, &sched_off, &st, now).is_err());
        // Exceptions can be removed but not added while the list is guarded.
        let mut excepted = old.clone();
        excepted.block_lists[0]
            .allow_sites
            .push("ok.example.com".into());
        assert!(validate_update(&old, &excepted, &st, now).is_err());
        let mut with_exc = old.clone();
        with_exc.block_lists[0].allow_sites = vec!["ok.example.com".into()];
        let mut removed = with_exc.clone();
        removed.block_lists[0].allow_sites.clear();
        assert!(validate_update(&with_exc, &removed, &st, now).is_ok());
        // "*" can never be an exception.
        let mut star = loose.clone();
        star.protection = Protection::default();
        star.block_lists[0].allow_sites = vec!["*".into()];
        assert!(validate_shape(&star).is_err());
        // A ready-made list can't lose its tag (it would stop getting updates).
        let mut tagged = old.clone();
        tagged.block_lists[0].preset = "adult".into();
        let mut untagged = tagged.clone();
        untagged.block_lists[0].preset = String::new();
        assert!(validate_update(&tagged, &untagged, &st, now).is_err());
        // Tightening is still fine.
        let mut more = old.clone();
        more.block_lists[0].sites.push("worse.com".into());
        assert!(validate_update(&old, &more, &st, now).is_ok());
    }
}
