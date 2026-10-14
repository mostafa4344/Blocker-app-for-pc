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
    #[error("session is locked")]
    SessionLocked { id: String },
    #[error("not found: {what}")]
    NotFound { what: String },
    #[error("invalid: {what}")]
    Invalid { what: String },
}

/// Block list ids that are currently locked.
pub fn locked_list_ids(config: &Config, state: &State, now: DateTime<Utc>) -> Vec<String> {
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
        .filter(|s| s.enabled && s.locked)
        .for_each(|s| push(&s.block_list_id));
    config
        .limits
        .iter()
        .filter(|l| l.enabled && l.locked)
        .for_each(|l| push(&l.block_list_id));
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
    if new.allowlist_mode != old.allowlist_mode {
        return false;
    }
    let sites_ok = if old.allowlist_mode {
        superset(&old.sites, &new.sites)
    } else {
        superset(&new.sites, &old.sites)
    };
    sites_ok && superset(&new.apps, &old.apps) && superset(&new.keywords, &old.keywords)
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
    for o in old.schedules.iter().filter(|s| s.enabled && s.locked) {
        let ok = new
            .schedules
            .iter()
            .find(|n| n.id == o.id)
            .is_some_and(|n| {
                n.enabled && n.locked && n.block_list_id == o.block_list_id && covers(n, o)
            });
        if !ok {
            return Err(PolicyError::ScheduleLocked { id: o.id.clone() });
        }
    }
    for o in old.limits.iter().filter(|l| l.enabled && l.locked) {
        let ok = new.limits.iter().find(|n| n.id == o.id).is_some_and(|n| {
            n.enabled
                && n.locked
                && n.block_list_id == o.block_list_id
                && n.daily_minutes <= o.daily_minutes
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
    if config.block_list(block_list_id).is_none() {
        return Err(PolicyError::NotFound {
            what: "block_list".into(),
        });
    }
    if !(1..=MAX_SESSION_MINUTES).contains(&minutes) {
        return Err(PolicyError::Invalid {
            what: "minutes".into(),
        });
    }
    if let LockKind::RandomText { length } = lock {
        if !(10..=2000).contains(&length) {
            return Err(PolicyError::Invalid {
                what: "lock.length".into(),
            });
        }
    }
    if lock == LockKind::Partner && config.protection.partner.is_none() {
        return Err(PolicyError::Invalid {
            what: "protection.partner".into(),
        });
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
        started_at: now,
        ends_at: now + Duration::minutes(minutes),
        lock,
    })
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

pub fn can_end_session(
    session: &Session,
    proof: UnlockProof,
    now: DateTime<Utc>,
) -> Result<(), PolicyError> {
    if !session.is_active(now) {
        return Ok(());
    }
    let ok = match session.lock {
        LockKind::None => true,
        LockKind::Timer => false,
        LockKind::RandomText { .. } => proof == UnlockProof::ChallengeSolved,
        LockKind::Restart => proof == UnlockProof::Restarted,
        LockKind::Partner => proof == UnlockProof::PartnerApproved,
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
        assert!(can_end_session(&s, UnlockProof::ChallengeSolved, now).is_err());
        assert!(can_end_session(&s, UnlockProof::Nothing, now + Duration::minutes(61)).is_ok());
        let r = start_session(&c, "a", 60, LockKind::RandomText { length: 100 }, now).unwrap();
        assert!(can_end_session(&r, UnlockProof::ChallengeSolved, now).is_ok());
        assert!(can_end_session(&r, UnlockProof::Restarted, now).is_err());
        assert!(start_session(&c, "a", 60, LockKind::Partner, now).is_err());
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
}
