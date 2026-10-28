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
    /// Version of the admin-managed preset lists the server last reported
    /// (empty if the server doesn't report one).
    pub presets_version: String,
    /// Version of the preset lists this install last downloaded.
    pub presets_fetched: String,
    /// Latest app release announced by the server (see `update.rs`).
    pub latest_release: Option<serde_json::Value>,
    /// Release version this device last tried to install, and how often.
    pub update_attempt_version: String,
    pub update_attempts: u32,
    /// The server said it accepts anonymous health reports with license checks.
    pub server_accepts_health: bool,
    /// Web page of the "Get Premium" button (set by the admin on the server).
    pub purchase_url: String,
    /// bKash/Nagad/Rocket numbers and prices set by the admin (app_config).
    pub pay: Option<serde_json::Value>,
    /// A purchase request sent from the app that the admin hasn't approved yet.
    pub pending_purchase: Option<serde_json::Value>,
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
            presets_version: String::new(),
            presets_fetched: String::new(),
            latest_release: None,
            update_attempt_version: String::new(),
            update_attempts: 0,
            server_accepts_health: false,
            purchase_url: String::new(),
            pay: None,
            pending_purchase: None,
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
    /// Changes whenever the admin edits `preset_lists` (newer servers only).
    presets_version: Option<String>,
    /// Newest enabled app release (newer servers only).
    latest_release: Option<serde_json::Value>,
    /// True on servers whose get_license takes `p_version` / `p_health`.
    accepts_health: Option<bool>,
    purchase_url: Option<String>,
    pay: Option<serde_json::Value>,
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
        .http_status_as_error(false)
        .build()
        .into()
}

/// Prefix of errors where the server answered but refused the request (as
/// opposed to network trouble). Used to show "invalid code" vs "offline".
pub const REJECTED: &str = "rejected: ";

fn call(ep: &Endpoint, func: &str, body: serde_json::Value) -> Result<ServerLicense, String> {
    let url = format!("{}/rest/v1/rpc/{func}", ep.base);
    let mut resp = agent()
        .post(&url)
        .header("apikey", &ep.key)
        .header("Authorization", &format!("Bearer {}", ep.key))
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        // PostgREST / the mock answer `{ "message": "..." }` on errors.
        let msg = resp
            .body_mut()
            .read_json::<serde_json::Value>()
            .ok()
            .and_then(|v| v["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| format!("http status: {status}"));
        return Err(if (400..500).contains(&status) {
            format!("{REJECTED}{msg}")
        } else {
            msg
        });
    }
    resp.body_mut()
        .read_json::<ServerLicense>()
        .map_err(|e| e.to_string())
}

/// The real time, from the `Date` header of the licensing server's HTTPS
/// answer (TLS means it can't be faked by changing anything on this PC).
/// Corrected for half the round trip.
pub fn server_time(ep: &Endpoint) -> Result<DateTime<Utc>, String> {
    let started = std::time::Instant::now();
    let resp = agent()
        .get(&format!("{}/rest/v1/", ep.base))
        .header("apikey", &ep.key)
        .call()
        .map_err(|e| e.to_string())?;
    let rtt = started.elapsed();
    let date = resp
        .headers()
        .get("date")
        .and_then(|v| v.to_str().ok())
        .ok_or("no Date header")?;
    let t = DateTime::parse_from_rfc2822(date.trim())
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    Ok(t + chrono::Duration::from_std(rtt / 2).unwrap_or_default())
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
    if let Some(v) = s.presets_version {
        lic.presets_version = v;
    }
    lic.latest_release = s.latest_release.filter(|v| v.is_object());
    lic.server_accepts_health = s.accepts_health.unwrap_or(false);
    if s.pay.as_ref().is_some_and(|p| p.is_object()) {
        lic.pay = s.pay;
    }
    // Only plain https links are accepted for the buy button.
    lic.purchase_url = s
        .purchase_url
        .filter(|u| u.starts_with("https://") && u.len() < 300 && !u.contains(char::is_whitespace))
        .unwrap_or_default();
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
pub fn refresh(
    ep: &Endpoint,
    lic: &mut License,
    now: DateTime<Utc>,
    version: &str,
    health: serde_json::Value,
) -> Result<(), String> {
    lic.ensure_install_id();
    // Older servers only know `p_install_id`; extra fields would be rejected.
    let body = if lic.server_accepts_health {
        serde_json::json!({ "p_install_id": lic.install_id, "p_version": version, "p_health": health })
    } else {
        serde_json::json!({ "p_install_id": lic.install_id })
    };
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

/// Fetches the ready-made block lists an admin maintains on the server
/// (`get_presets` RPC over the `preset_lists` table). Callers must sanitize.
pub fn fetch_presets(ep: &Endpoint) -> Result<Vec<contentguard_core::presets::PresetDef>, String> {
    let url = format!("{}/rest/v1/rpc/get_presets", ep.base);
    let mut resp = agent()
        .post(&url)
        .header("apikey", &ep.key)
        .header("Authorization", &format!("Bearer {}", ep.key))
        .header("Content-Type", "application/json")
        .send_json(serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    if !(200..300).contains(&resp.status().as_u16()) {
        return Err(format!("http status: {}", resp.status()));
    }
    let v: Option<Vec<contentguard_core::presets::PresetDef>> =
        resp.body_mut().read_json().map_err(|e| e.to_string())?;
    Ok(v.unwrap_or_default())
}

/// Sends a bKash/Nagad/Rocket purchase (plan + TrxID) for the admin to approve.
pub fn submit_purchase(
    ep: &Endpoint,
    install_id: &str,
    plan: &str,
    method: &str,
    txn_id: &str,
    contact: &str,
) -> Result<(), String> {
    let url = format!("{}/rest/v1/rpc/submit_purchase_request", ep.base);
    let mut resp = agent()
        .post(&url)
        .header("apikey", &ep.key)
        .header("Authorization", &format!("Bearer {}", ep.key))
        .header("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "p_install_id": install_id, "p_plan": plan, "p_method": method,
            "p_txn_id": txn_id, "p_contact": contact,
        }))
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if (200..300).contains(&status) {
        return Ok(());
    }
    let msg = resp
        .body_mut()
        .read_json::<serde_json::Value>()
        .ok()
        .and_then(|v| v["message"].as_str().map(str::to_string))
        .unwrap_or_else(|| format!("http status: {status}"));
    Err(if (400..500).contains(&status) {
        format!("{REJECTED}{msg}")
    } else {
        msg
    })
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
