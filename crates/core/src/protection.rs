//! Protection requests: the only way to uninstall, lower the protection level
//! or end a partner-locked session. Every level always has a legitimate way
//! out (a cooldown or a partner timeout) so nobody is locked in permanently.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{Protection, ProtectionLevel};

/// Mindful level: short pause after the challenge has been typed.
pub const MINDFUL_WAIT_SECS: i64 = 60;
/// How long an approved request can be used before it expires.
pub const APPROVAL_VALID_HOURS: i64 = 24;
/// Upper bound for the Committed cooldown (~100 years). The UI presents the
/// largest option as "unlimited"; a finite bound is kept so a request can never
/// become literally un-resolvable (see NOTES.md on the legal/anti-malware reason).
pub const MAX_COOLDOWN_HOURS: u32 = 876_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestKind {
    Uninstall,
    /// Replace the protection settings with less strict ones.
    ChangeProtection {
        new: Protection,
    },
    /// End a partner-locked session early.
    EndSession {
        session_id: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Waiting for the challenge (Mindful).
    NeedsChallenge,
    /// Waiting for the cooldown or the partner.
    Pending,
    Approved,
    Denied,
    Cancelled,
    /// Approved but not used in time.
    Expired,
    /// Approved and acted upon.
    Used,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: String,
    pub kind: RequestKind,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    /// Earliest time the request becomes approved on its own (cooldown or partner timeout).
    pub available_at: Option<DateTime<Utc>>,
    pub needs_partner: bool,
    pub status: RequestStatus,
    pub decided_at: Option<DateTime<Utc>>,
}

impl Request {
    pub fn new(
        protection: &Protection,
        kind: RequestKind,
        reason: &str,
        now: DateTime<Utc>,
    ) -> Request {
        let mut req = Request {
            id: crate::new_id(),
            kind,
            reason: reason.trim().to_string(),
            created_at: now,
            available_at: None,
            needs_partner: false,
            status: RequestStatus::Pending,
            decided_at: None,
        };
        match protection.level {
            ProtectionLevel::Off => {
                req.status = RequestStatus::Approved;
                req.decided_at = Some(now);
            }
            ProtectionLevel::Mindful => req.status = RequestStatus::NeedsChallenge,
            ProtectionLevel::Committed => {
                req.available_at = Some(
                    now + Duration::hours(
                        protection.cooldown_hours.clamp(1, MAX_COOLDOWN_HOURS) as i64
                    ),
                );
            }
            ProtectionLevel::Partner => {
                req.needs_partner = protection.partner.is_some();
                let days = if req.needs_partner {
                    protection.partner_timeout_days.clamp(1, 30)
                } else {
                    3
                };
                req.available_at = Some(now + Duration::days(days as i64));
            }
        }
        req
    }

    /// Called once the Mindful challenge has been solved.
    pub fn challenge_solved(&mut self, now: DateTime<Utc>) {
        if self.status == RequestStatus::NeedsChallenge {
            self.status = RequestStatus::Pending;
            self.available_at = Some(now + Duration::seconds(MINDFUL_WAIT_SECS));
        }
    }

    pub fn partner_decision(&mut self, approve: bool, now: DateTime<Utc>) {
        if self.status == RequestStatus::Pending && self.needs_partner {
            self.status = if approve {
                RequestStatus::Approved
            } else {
                RequestStatus::Denied
            };
            self.decided_at = Some(now);
        }
    }

    pub fn cancel(&mut self, now: DateTime<Utc>) {
        if matches!(
            self.status,
            RequestStatus::Pending | RequestStatus::NeedsChallenge | RequestStatus::Approved
        ) {
            self.status = RequestStatus::Cancelled;
            self.decided_at = Some(now);
        }
    }

    /// Advances time-based transitions (cooldown elapsed, partner timeout, expiry).
    pub fn tick(&mut self, now: DateTime<Utc>) {
        if self.status == RequestStatus::Pending {
            if let Some(at) = self.available_at {
                if now >= at {
                    self.status = RequestStatus::Approved;
                    self.decided_at = Some(at);
                }
            }
        }
        if self.status == RequestStatus::Approved {
            if let Some(d) = self.decided_at {
                if now >= d + Duration::hours(APPROVAL_VALID_HOURS) {
                    self.status = RequestStatus::Expired;
                }
            }
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(
            self.status,
            RequestStatus::NeedsChallenge | RequestStatus::Pending | RequestStatus::Approved
        )
    }
}

/// Whether `new` is at least as strict as `old` (so it can be applied without a request).
pub fn is_at_least_as_strict(new: &Protection, old: &Protection) -> bool {
    if new.level < old.level {
        return false;
    }
    if new.level == old.level {
        match new.level {
            ProtectionLevel::Committed if new.cooldown_hours < old.cooldown_hours => return false,
            ProtectionLevel::Partner => {
                if old.partner.is_some() && new.partner != old.partner {
                    return false;
                }
                if new.partner_timeout_days < old.partner_timeout_days {
                    return false;
                }
            }
            _ => {}
        }
    }
    if old.tamper_alerts && !new.tamper_alerts && old.level >= ProtectionLevel::Committed {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Partner;

    fn prot(level: ProtectionLevel) -> Protection {
        Protection {
            level,
            partner: Some(Partner {
                name: "A".into(),
                email: "a@example.com".into(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn off_is_immediate() {
        let r = Request::new(
            &prot(ProtectionLevel::Off),
            RequestKind::Uninstall,
            "",
            Utc::now(),
        );
        assert_eq!(r.status, RequestStatus::Approved);
    }

    #[test]
    fn mindful_flow() {
        let now = Utc::now();
        let mut r = Request::new(
            &prot(ProtectionLevel::Mindful),
            RequestKind::Uninstall,
            "",
            now,
        );
        r.tick(now + Duration::hours(1));
        assert_eq!(r.status, RequestStatus::NeedsChallenge);
        r.challenge_solved(now);
        r.tick(now + Duration::seconds(30));
        assert_eq!(r.status, RequestStatus::Pending);
        r.tick(now + Duration::seconds(61));
        assert_eq!(r.status, RequestStatus::Approved);
    }

    #[test]
    fn committed_cooldown_and_expiry() {
        let now = Utc::now();
        let mut r = Request::new(
            &prot(ProtectionLevel::Committed),
            RequestKind::Uninstall,
            "",
            now,
        );
        r.tick(now + Duration::hours(23));
        assert_eq!(r.status, RequestStatus::Pending);
        r.tick(now + Duration::hours(24));
        assert_eq!(r.status, RequestStatus::Approved);
        r.tick(now + Duration::hours(49));
        assert_eq!(r.status, RequestStatus::Expired);
    }

    #[test]
    fn partner_approve_deny_timeout() {
        let now = Utc::now();
        let p = prot(ProtectionLevel::Partner);
        let mut a = Request::new(&p, RequestKind::Uninstall, "", now);
        assert!(a.needs_partner);
        a.partner_decision(true, now);
        assert_eq!(a.status, RequestStatus::Approved);

        let mut d = Request::new(&p, RequestKind::Uninstall, "", now);
        d.partner_decision(false, now);
        d.tick(now + Duration::days(30));
        assert_eq!(d.status, RequestStatus::Denied);

        let mut t = Request::new(&p, RequestKind::Uninstall, "", now);
        t.tick(now + Duration::days(7));
        assert_eq!(t.status, RequestStatus::Approved);
    }

    #[test]
    fn strictness() {
        let off = Protection::default();
        let committed = prot(ProtectionLevel::Committed);
        assert!(is_at_least_as_strict(&committed, &off));
        assert!(!is_at_least_as_strict(&off, &committed));
        let mut shorter = committed.clone();
        shorter.cooldown_hours = 1;
        assert!(!is_at_least_as_strict(&shorter, &committed));
        let mut other_partner = prot(ProtectionLevel::Partner);
        other_partner.partner = Some(Partner {
            name: "B".into(),
            email: "b@example.com".into(),
        });
        assert!(!is_at_least_as_strict(
            &other_partner,
            &prot(ProtectionLevel::Partner)
        ));
    }
}
