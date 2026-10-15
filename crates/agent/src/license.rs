//! Licensing: a free trial, then a paid plan. The source of truth is the
//! backend (a Supabase project via RPC, or any compatible server). Status is
//! cached locally so the app keeps working offline, but premium is only trusted
//! for a bounded time without a successful recheck, to resist offline abuse.
//!
//! Security note: the trial is anchored server-side to a persistent `install_id`,
//! so clearing local data does not reset it once the device has been seen online.
//! The `install_id` is a per-install secret; it is the bearer credential for the
//! account. See NOTES.md for the known limits and the planned payment wiring.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Default free-trial length when the backend does not state one.
pub const DEFAULT_TRIAL_DAYS: i64 = 4;
/// How long a cached "premium" is trusted with no successful server recheck.
pub const OFFLINE_GRACE_DAYS: i64 = 14;

/// Built-in backend used when the user has not configured one. The publishable
/// (anon) key is designed to be shipped in clients; real authority is enforced
/// server-side by RLS and the RPC functions in `supabase/`.
pub const DEFAULT_BACKEND_URL: &str = "https://qhplhhrmathbfyejkzbr.supabase.co";
pub const DEFAULT_BACKEND_KEY: &str = "sb_publishable_j4Ogmxj4LN9SM8J2ovPEiw_2T--cmfD";

/// Cached licensing state (persisted to `license.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct License {
    /// Persistent per-install secret and account identifier.
    pub install_id: String,
    /// Server plan: "trial", "premium" or "expired" (empty before first contact).
    pub plan: String,
    pub trial_started: Option<DateTime<Utc>>,
    pub trial_days: i64,
    /// Premium end; `None` with plan "premium" means perpetual.
    pub premium_until: Option<DateTime<Utc>>,
    /// Last successful server exchange.
    pub last_checked: Option<DateTime<Utc>>,
    /// Last error message from the backend, for display.
    pub last_error: String,
}

impl Default for License {
    fn default() -> Self {
        License {
            install_id: String::new(),
            plan: String::new(),
            trial_started: None,
            trial_days: DEFAULT_TRIAL_DAYS,
            premium_until: None,
            last_checked: None,
            last_error: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    /// Trial running; `days_left` is rounded up.
    Trial {
        days_left: i64,
    },
    Premium,
    /// Trial ended and no active premium.
    Expired,
    /// Was premium but has not reached the backend within the offline grace.
    LockedOffline,
}

impl License {
    pub fn ensure_install_id(&mut self) {
        if self.install_id.trim().is_empty() {
            self.install_id = format!(
                "{}{}",
                contentguard_core::new_id(),
                contentguard_core::new_id()
            );
        }
    }

    fn trial_active(&self, now: DateTime<Utc>) -> Option<i64> {
        let started = self.trial_started?;
        let end = started + Duration::days(self.trial_days.max(0));
        if now < end {
            // Days remaining, rounded up (a part-day still counts as a day).
            Some(
                (end - now)
                    .num_seconds()
                    .saturating_add(86_399)
                    .div_euclid(86_400),
            )
        } else {
            None
        }
    }

    /// Premium by plan and (for subscriptions) still within the paid period —
    /// before applying the offline-freshness requirement.
    fn premium_paid(&self, now: DateTime<Utc>) -> bool {
        self.plan == "premium" && self.premium_until.map(|u| now < u).unwrap_or(true)
    }

    pub fn status(&self, now: DateTime<Utc>) -> Status {
        if self.premium_paid(now) {
            // Local state can be forged, so even lifetime premium must be
            // re-confirmed with the backend within the offline grace window.
            let fresh = self
                .last_checked
                .map(|t| now - t < Duration::days(OFFLINE_GRACE_DAYS))
                .unwrap_or(false);
            if fresh {
                return Status::Premium;
            }
            // Stale premium: fall back to the trial if any remains, else lock.
            return match self.trial_active(now) {
                Some(days_left) => Status::Trial { days_left },
                None => Status::LockedOffline,
            };
        }
        match self.trial_active(now) {
            Some(days_left) => Status::Trial { days_left },
            None => Status::Expired,
        }
    }

    /// Whether the user may add or strengthen blocking right now.
    pub fn allows_management(&self, now: DateTime<Utc>) -> bool {
        matches!(self.status(now), Status::Trial { .. } | Status::Premium)
    }
}

// ---------- backend exchange ----------

#[derive(Deserialize)]
struct ServerLicense {
    plan: Option<String>,
    trial_started: Option<DateTime<Utc>>,
    trial_days: Option<i64>,
    premium_until: Option<DateTime<Utc>>,
}

pub struct Endpoint {
    pub base: String,
    pub key: String,
}

impl Endpoint {
    /// Resolves the configured backend, falling back to the built-in default.
    pub fn resolve(url: &str, key: &str) -> Endpoint {
        let base = if url.trim().is_empty() {
            DEFAULT_BACKEND_URL
        } else {
            url.trim()
        };
        let key = if key.trim().is_empty() {
            DEFAULT_BACKEND_KEY
        } else {
            key.trim()
        };
        Endpoint {
            base: base.trim_end_matches('/').to_string(),
            key: key.to_string(),
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(StdDuration::from_secs(15)))
        .http_status_as_error(true)
        .build()
        .into()
}

fn call(ep: &Endpoint, func: &str, body: serde_json::Value) -> Result<ServerLicense, String> {
    let url = format!("{}/rest/v1/rpc/{func}", ep.base);
    let resp = agent()
        .post(&url)
        .header("apikey", &ep.key)
        .header("Authorization", &format!("Bearer {}", ep.key))
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json::<ServerLicense>()
        .map_err(|e| e.to_string())?;
    Ok(resp)
}

fn apply(lic: &mut License, s: ServerLicense, now: DateTime<Utc>) {
    if let Some(p) = s.plan {
        lic.plan = p;
    }
    if s.trial_started.is_some() {
        lic.trial_started = s.trial_started;
    }
    if let Some(d) = s.trial_days {
        lic.trial_days = d;
    }
    lic.premium_until = s.premium_until;
    lic.last_checked = Some(now);
    lic.last_error.clear();
}

fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// Registers the install (creating the account + trial on first contact) and
/// refreshes the cached status. Safe to call repeatedly.
pub fn register(
    ep: &Endpoint,
    lic: &mut License,
    version: &str,
    now: DateTime<Utc>,
) -> Result<(), String> {
    lic.ensure_install_id();
    let body = serde_json::json!({
        "p_install_id": lic.install_id,
        "p_os": os_name(),
        "p_version": version,
        "p_trial_days": DEFAULT_TRIAL_DAYS,
    });
    match call(ep, "register_device", body) {
        Ok(s) => {
            apply(lic, s, now);
            Ok(())
        }
        Err(e) => {
            lic.last_error = e.clone();
            Err(e)
        }
    }
}

/// Refreshes cached status from the backend.
pub fn refresh(ep: &Endpoint, lic: &mut License, now: DateTime<Utc>) -> Result<(), String> {
    lic.ensure_install_id();
    let body = serde_json::json!({ "p_install_id": lic.install_id });
    match call(ep, "get_license", body) {
        Ok(s) => {
            apply(lic, s, now);
            Ok(())
        }
        Err(e) => {
            lic.last_error = e.clone();
            Err(e)
        }
    }
}

/// Redeems a premium activation code bought by the user.
pub fn redeem(
    ep: &Endpoint,
    lic: &mut License,
    code: &str,
    now: DateTime<Utc>,
) -> Result<(), String> {
    lic.ensure_install_id();
    let body = serde_json::json!({ "p_install_id": lic.install_id, "p_code": code.trim() });
    match call(ep, "redeem_code", body) {
        Ok(s) => {
            apply(lic, s, now);
            Ok(())
        }
        Err(e) => {
            lic.last_error = e.clone();
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(days_from_now: i64, base: DateTime<Utc>) -> DateTime<Utc> {
        base + Duration::days(days_from_now)
    }

    #[test]
    fn trial_countdown_then_expiry() {
        let now = Utc::now();
        let mut lic = License {
            trial_started: Some(now),
            trial_days: 4,
            ..Default::default()
        };
        assert_eq!(lic.status(now), Status::Trial { days_left: 4 });
        assert!(lic.allows_management(now));
        assert_eq!(lic.status(at(3, now)), Status::Trial { days_left: 1 });
        assert_eq!(lic.status(at(4, now)), Status::Expired);
        assert!(!lic.allows_management(at(5, now)));
        lic.install_id.clear();
        lic.ensure_install_id();
        assert!(!lic.install_id.is_empty());
    }

    #[test]
    fn premium_perpetual_and_timed() {
        let now = Utc::now();
        let mut lic = License {
            plan: "premium".into(),
            premium_until: None,
            last_checked: Some(now),
            ..Default::default()
        };
        assert_eq!(lic.status(now), Status::Premium);
        // Even lifetime premium must re-confirm within the offline grace window.
        assert_eq!(lic.status(at(400, now)), Status::LockedOffline);

        lic.premium_until = Some(at(30, now));
        assert_eq!(lic.status(now), Status::Premium);
        assert_eq!(lic.status(at(31, now)), Status::Expired);
    }

    #[test]
    fn offline_grace_locks_subscription() {
        let now = Utc::now();
        // Subscription with no end date but checked long ago, and no trial left.
        let lic = License {
            plan: "premium".into(),
            premium_until: None,
            last_checked: Some(now - Duration::days(OFFLINE_GRACE_DAYS + 1)),
            ..Default::default()
        };
        assert_eq!(lic.status(now), Status::LockedOffline);
        assert!(!lic.allows_management(now));
    }
}
