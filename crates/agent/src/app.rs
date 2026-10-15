//! Shared agent state and every operation the API exposes. The agent is the
//! single authority: the UI and the extension only send requests, and every
//! change is validated here against the lock rules.

use std::collections::HashMap;
use std::io;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use serde_json::{json, Value};

use contentguard_core::challenge::{self, Challenge};
use contentguard_core::clock::TrustedClock;
use contentguard_core::engine::{self, Now, Policy};
use contentguard_core::lock::{self, PolicyError, UnlockProof};
use contentguard_core::model::{Config, LockKind, Protection, ProtectionLevel, State};
use contentguard_core::presets::{self, PRESETS};
use contentguard_core::protection::{Request, RequestKind, RequestStatus};
use contentguard_core::store::Store;

use crate::license::{self, License};
use crate::processes::{SUPPORTED_BROWSERS, UNSUPPORTED_BROWSERS};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_FILE: &str = "config.json";
const STATE_FILE: &str = "state.json";
const LICENSE_FILE: &str = "license.json";
/// Length of the challenge for Mindful-level protection requests.
const REQUEST_CHALLENGE_LENGTH: usize = 150;
const PRACTICE_DAYS: i64 = 7;
const STATS_KEEP_DAYS: i64 = 35;

pub struct Inner {
    pub config: Config,
    pub state: State,
    pub license: License,
    pub clock: TrustedClock,
    last_tick: Instant,
    /// Browser family -> last time its extension reported in.
    pub extension_seen: HashMap<String, Instant>,
    pub hosts_status: Result<(), String>,
    pub state_dirty: bool,
}

pub struct App {
    store: Store,
    inner: Mutex<Inner>,
    /// When the computer last started (for "unlock after restart").
    pub boot_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub body: Value,
}

impl ApiError {
    pub fn new(status: u16, code: &str, message: impl Into<String>) -> Self {
        ApiError {
            status,
            body: json!({ "code": code, "message": message.into() }),
        }
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(400, "bad_request", msg)
    }
    pub fn not_found(what: &str) -> Self {
        Self::new(404, "not_found", what)
    }
}

impl From<PolicyError> for ApiError {
    fn from(e: PolicyError) -> Self {
        let status = match e {
            PolicyError::NotFound { .. } => 404,
            PolicyError::Invalid { .. } => 400,
            _ => 409,
        };
        let mut body = serde_json::to_value(&e).unwrap_or(json!({}));
        body["message"] = json!(e.to_string());
        ApiError { status, body }
    }
}

impl From<io::Error> for ApiError {
    fn from(e: io::Error) -> Self {
        ApiError::new(500, "io", e.to_string())
    }
}

pub type ApiResult = Result<Value, ApiError>;

pub fn local_now(utc: DateTime<Utc>) -> Now {
    Now {
        utc,
        local: Local.from_utc_datetime(&utc.naive_utc()).naive_local(),
    }
}

impl App {
    pub fn open(store: Store) -> io::Result<App> {
        let config: Config = store.load(CONFIG_FILE)?;
        let state: State = store.load(STATE_FILE)?;
        let mut license: License = store.load(LICENSE_FILE)?;
        license.ensure_install_id();
        // Start the trial immediately (and persist it) so the app is usable on
        // first run before the backend is reached; the server corrects it on sync.
        if license.trial_started.is_none() {
            license.trial_started = Some(Utc::now());
            let _ = store.save(LICENSE_FILE, &license);
        }
        let clock = TrustedClock::new(Utc::now(), state.trusted_now);
        let boot_time = Utc
            .timestamp_opt(sysinfo::System::boot_time() as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);
        Ok(App {
            store,
            inner: Mutex::new(Inner {
                config,
                state,
                license,
                clock,
                last_tick: Instant::now(),
                extension_seen: HashMap::new(),
                hosts_status: Ok(()),
                state_dirty: false,
            }),
            boot_time,
        })
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        // A panic while holding the lock must not take the whole agent down.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn save_state(&self, inner: &mut Inner) -> io::Result<()> {
        inner.state.trusted_now = Some(inner.clock.now());
        inner.state_dirty = false;
        self.store.save(STATE_FILE, &inner.state)
    }

    pub fn save_config(&self, inner: &Inner) -> io::Result<()> {
        self.store.save(CONFIG_FILE, &inner.config)
    }

    pub fn save_license(&self, inner: &Inner) -> io::Result<()> {
        self.store.save(LICENSE_FILE, &inner.license)
    }

    /// The licensing endpoint resolved from config (falls back to the built-in default).
    pub fn license_endpoint(&self, inner: &Inner) -> license::Endpoint {
        license::Endpoint::resolve(
            &inner.config.settings.backend_url,
            &inner.config.settings.backend_key,
        )
    }

    /// Rejects an add/strengthen action when the user has no active plan.
    fn require_license(&self, inner: &Inner, now: DateTime<Utc>) -> Result<(), ApiError> {
        if inner.license.allows_management(now) {
            Ok(())
        } else {
            let st = inner.license.status(now);
            Err(ApiError {
                status: 402,
                body: json!({
                    "code": "payment_required",
                    "message": "Premium required to add or strengthen blocking.",
                    "license": st,
                }),
            })
        }
    }

    /// Advances the trusted clock and time-based request transitions.
    /// Returns the current time and the effective policy.
    pub fn advance(&self, inner: &mut Inner) -> (Now, Policy) {
        let elapsed = inner.last_tick.elapsed();
        inner.last_tick = Instant::now();
        let utc = inner.clock.tick(Utc::now(), elapsed);
        let now = local_now(utc);
        for r in inner.state.requests.iter_mut() {
            let before = r.status;
            r.tick(utc);
            if r.status != before {
                inner.state_dirty = true;
            }
        }
        let before = inner.state.sessions.len();
        engine::prune(&mut inner.state, now, STATS_KEEP_DAYS);
        if inner.state.sessions.len() != before {
            inner.state_dirty = true;
        }
        let policy = engine::effective_policy(&inner.config, &inner.state, now);
        (now, policy)
    }

    fn now(&self, inner: &mut Inner) -> Now {
        self.advance(inner).0
    }

    // ----- Read -----

    pub fn status(&self) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let (now, policy) = self.advance(inner);
        let day = now.day();
        let usage_today: Vec<&contentguard_core::model::Usage> =
            inner.state.usage.iter().filter(|u| u.day == day).collect();
        let mut stats_today: Vec<&contentguard_core::model::DailyStat> =
            inner.state.stats.iter().filter(|s| s.day == day).collect();
        stats_today.sort_by_key(|s| std::cmp::Reverse(s.seconds));
        stats_today.truncate(15);
        let mut week: Vec<Value> = Vec::new();
        for d in (0..7).rev() {
            let key = (now.local.date() - Duration::days(d))
                .format("%Y-%m-%d")
                .to_string();
            let (secs, blocked) = inner
                .state
                .stats
                .iter()
                .filter(|s| s.day == key)
                .fold((0u64, 0u32), |(a, b), s| {
                    (a + s.seconds, b + s.blocked_attempts)
                });
            week.push(json!({ "day": key, "seconds": secs, "blocked_attempts": blocked }));
        }
        let ext: HashMap<&String, u64> = inner
            .extension_seen
            .iter()
            .map(|(k, v)| (k, v.elapsed().as_secs()))
            .collect();
        let sessions: Vec<_> = inner
            .state
            .sessions
            .iter()
            .filter(|s| s.ends_at > now.utc)
            .collect();
        let practice = inner
            .config
            .settings
            .practice_mode_until
            .is_some_and(|t| now.utc < t);
        Ok(json!({
            "version": VERSION,
            "now": now.utc,
            "clock_skew_seconds": inner.clock.skew().num_seconds(),
            "config": inner.config,
            "policy": policy,
            "sessions": sessions,
            "requests": inner.state.requests,
            "challenge": inner.state.challenge.as_ref().map(|c| json!({ "id": c.id, "purpose": c.purpose, "expires_at": c.expires_at })),
            "usage_today": usage_today,
            "stats_today": stats_today,
            "week": week,
            "locked_list_ids": lock::locked_list_ids(&inner.config, &inner.state, now.utc),
            "hosts": match &inner.hosts_status { Ok(()) => json!({ "ok": true }), Err(e) => json!({ "ok": false, "error": e }) },
            "extension_seen_seconds_ago": ext,
            "boot_time": self.boot_time,
            "practice_mode": practice,
            "presets": PRESETS,
            "license": {
                "status": inner.license.status(now.utc),
                "plan": inner.license.plan,
                "allows_management": inner.license.allows_management(now.utc),
                "trial_days": inner.license.trial_days,
                "premium_until": inner.license.premium_until,
                "last_checked": inner.license.last_checked,
                "last_error": inner.license.last_error,
            },
            "browsers": {
                "supported": SUPPORTED_BROWSERS,
                "unsupported": UNSUPPORTED_BROWSERS,
            },
        }))
    }

    // ----- Config -----

    pub fn update_config(&self, mut new: Config) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        // Fields the agent owns.
        new.version = contentguard_core::model::CONFIG_VERSION;
        new.settings.practice_mode_until = inner.config.settings.practice_mode_until;
        if new.settings.onboarded
            && !inner.config.settings.onboarded
            && new.settings.practice_mode_until.is_none()
        {
            new.settings.practice_mode_until = Some(now.utc + Duration::days(PRACTICE_DAYS));
        }
        lock::validate_update(&inner.config, &new, &inner.state, now.utc)?;
        // Without an active plan, changes may only remove or weaken blocking.
        if block_weight(&new) > block_weight(&inner.config) {
            self.require_license(inner, now.utc)?;
        }
        inner.config = new;
        self.save_config(inner)?;
        Ok(json!({ "ok": true, "config": inner.config }))
    }

    pub fn add_preset(&self, id: &str) -> ApiResult {
        let preset = presets::preset(id).ok_or_else(|| ApiError::not_found("preset"))?;
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        self.require_license(inner, now.utc)?;
        let list = preset.to_block_list(&inner.config.settings.language);
        let list_id = list.id.clone();
        inner.config.block_lists.push(list);
        self.save_config(inner)?;
        Ok(json!({ "ok": true, "block_list_id": list_id }))
    }

    /// Ends practice mode early (locks can then be longer than two hours).
    pub fn end_practice(&self) -> ApiResult {
        let mut g = self.lock();
        g.config.settings.practice_mode_until = None;
        self.save_config(&g)?;
        Ok(json!({ "ok": true }))
    }

    // ----- Sessions -----

    pub fn start_session(
        &self,
        block_list_id: &str,
        minutes: i64,
        lock_kind: LockKind,
    ) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        self.require_license(inner, now.utc)?;
        let s = lock::start_session(&inner.config, block_list_id, minutes, lock_kind, now.utc)?;
        let v = json!(s);
        inner.state.sessions.push(s);
        self.save_state(inner)?;
        Ok(json!({ "ok": true, "session": v }))
    }

    pub fn extend_session(&self, id: &str, minutes: i64) -> ApiResult {
        if !(1..=lock::MAX_SESSION_MINUTES).contains(&minutes) {
            return Err(ApiError::bad_request("minutes"));
        }
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        self.require_license(inner, now.utc)?;
        let s = inner
            .state
            .sessions
            .iter_mut()
            .find(|s| s.id == id && s.ends_at > now.utc)
            .ok_or_else(|| ApiError::not_found("session"))?;
        s.ends_at += Duration::minutes(minutes);
        let max_end = now.utc + Duration::minutes(lock::MAX_SESSION_MINUTES);
        if s.ends_at > max_end {
            s.ends_at = max_end;
        }
        let v = json!(s);
        self.save_state(inner)?;
        Ok(json!({ "ok": true, "session": v }))
    }

    pub fn stop_session(&self, id: &str, answer: Option<&str>) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        let idx = inner
            .state
            .sessions
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| ApiError::not_found("session"))?;
        let session = inner.state.sessions[idx].clone();
        let proof = match (&session.lock, answer) {
            (LockKind::RandomText { .. }, Some(a)) => {
                let purpose = format!("session:{id}");
                match &inner.state.challenge {
                    Some(c) if challenge::verify(c, &purpose, a, now.utc) => {
                        UnlockProof::ChallengeSolved
                    }
                    _ => {
                        return Err(ApiError::new(
                            409,
                            "challenge_failed",
                            "challenge text does not match",
                        ))
                    }
                }
            }
            (LockKind::Restart, _) if self.boot_time > session.started_at => UnlockProof::Restarted,
            (LockKind::Partner, _) => {
                let approved = inner.state.requests.iter_mut().find(|r| {
                    r.status == RequestStatus::Approved
                        && matches!(&r.kind, RequestKind::EndSession { session_id } if session_id == id)
                });
                match approved {
                    Some(r) => {
                        r.status = RequestStatus::Used;
                        UnlockProof::PartnerApproved
                    }
                    None => UnlockProof::Nothing,
                }
            }
            _ => UnlockProof::Nothing,
        };
        lock::can_end_session(&session, proof, now.utc)?;
        inner.state.sessions.remove(idx);
        if proof == UnlockProof::ChallengeSolved {
            inner.state.challenge = None;
        }
        self.save_state(inner)?;
        Ok(json!({ "ok": true }))
    }

    // ----- Challenges -----

    pub fn new_challenge(&self, purpose: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        let length = if let Some(id) = purpose.strip_prefix("session:") {
            match inner
                .state
                .sessions
                .iter()
                .find(|s| s.id == id)
                .map(|s| &s.lock)
            {
                Some(LockKind::RandomText { length }) => *length,
                _ => return Err(ApiError::bad_request("session has no text challenge")),
            }
        } else if let Some(id) = purpose.strip_prefix("request:") {
            match inner.state.requests.iter().find(|r| r.id == id) {
                Some(r) if r.status == RequestStatus::NeedsChallenge => REQUEST_CHALLENGE_LENGTH,
                _ => return Err(ApiError::bad_request("request does not need a challenge")),
            }
        } else {
            return Err(ApiError::bad_request("purpose"));
        };
        let c: Challenge = challenge::generate(length, purpose, now.utc);
        let v =
            json!({ "id": c.id, "purpose": c.purpose, "text": c.text, "expires_at": c.expires_at });
        inner.state.challenge = Some(c);
        self.save_state(inner)?;
        Ok(v)
    }

    // ----- Protection requests -----

    pub fn create_request(&self, kind: RequestKind, reason: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        match &kind {
            RequestKind::ChangeProtection { new } => {
                let mut probe = inner.config.clone();
                probe.protection = new.clone();
                lock::validate_shape(&probe)?;
            }
            RequestKind::EndSession { session_id } => {
                let s = inner
                    .state
                    .sessions
                    .iter()
                    .find(|s| &s.id == session_id && s.ends_at > now.utc);
                if !s.is_some_and(|s| s.lock == LockKind::Partner) {
                    return Err(ApiError::bad_request(
                        "only partner-locked sessions use requests",
                    ));
                }
            }
            RequestKind::Uninstall => {}
        }
        if inner
            .state
            .requests
            .iter()
            .any(|r| r.is_open() && r.kind == kind)
        {
            return Err(ApiError::new(
                409,
                "duplicate",
                "a request like this is already open",
            ));
        }
        // Partner-locked sessions always need the partner, whatever the protection level.
        let protection = match kind {
            RequestKind::EndSession { .. } => Protection {
                level: ProtectionLevel::Partner,
                ..inner.config.protection.clone()
            },
            _ => inner.config.protection.clone(),
        };
        let r = Request::new(&protection, kind, reason, now.utc);
        let v = json!(r);
        inner.state.requests.push(r);
        self.save_state(inner)?;
        Ok(json!({ "ok": true, "request": v }))
    }

    pub fn solve_request_challenge(&self, id: &str, answer: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        let purpose = format!("request:{id}");
        let ok = inner
            .state
            .challenge
            .as_ref()
            .is_some_and(|c| challenge::verify(c, &purpose, answer, now.utc));
        if !ok {
            return Err(ApiError::new(
                409,
                "challenge_failed",
                "challenge text does not match",
            ));
        }
        let r = inner
            .state
            .requests
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| ApiError::not_found("request"))?;
        r.challenge_solved(now.utc);
        let v = json!(r);
        inner.state.challenge = None;
        self.save_state(inner)?;
        Ok(json!({ "ok": true, "request": v }))
    }

    pub fn cancel_request(&self, id: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        let r = inner
            .state
            .requests
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| ApiError::not_found("request"))?;
        r.cancel(now.utc);
        self.save_state(inner)?;
        Ok(json!({ "ok": true }))
    }

    /// Carries out an approved request (uninstall approval is consumed by `uninstall_check`).
    pub fn apply_request(&self, id: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        self.now(inner);
        let idx = inner
            .state
            .requests
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| ApiError::not_found("request"))?;
        if inner.state.requests[idx].status != RequestStatus::Approved {
            return Err(ApiError::new(
                409,
                "not_approved",
                "request is not approved",
            ));
        }
        match inner.state.requests[idx].kind.clone() {
            RequestKind::ChangeProtection { new } => {
                inner.config.protection = new;
                self.save_config(inner)?;
            }
            RequestKind::EndSession { session_id } => {
                inner.state.sessions.retain(|s| s.id != session_id);
            }
            RequestKind::Uninstall => {
                return Ok(
                    json!({ "ok": true, "message": "approved: run the uninstaller within 24 hours" }),
                );
            }
        }
        inner.state.requests[idx].status = RequestStatus::Used;
        self.save_state(inner)?;
        Ok(json!({ "ok": true }))
    }

    /// Used by the uninstaller. Returns true (and consumes the approval) when uninstalling is allowed.
    pub fn uninstall_check(&self) -> io::Result<bool> {
        let mut g = self.lock();
        let inner = &mut *g;
        self.now(inner);
        if inner.config.protection.level == ProtectionLevel::Off {
            return Ok(true);
        }
        let approved = inner
            .state
            .requests
            .iter_mut()
            .find(|r| r.kind == RequestKind::Uninstall && r.status == RequestStatus::Approved);
        match approved {
            Some(r) => {
                r.status = RequestStatus::Used;
                self.save_state(inner)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // ----- Browser extension -----

    pub fn extension_policy(&self, family: &str) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        inner
            .extension_seen
            .insert(family.to_string(), Instant::now());
        let (_, policy) = self.advance(inner);
        Ok(json!({
            "blocked_sites": policy.blocked_sites,
            "keywords": policy.keywords,
            "allowlist_active": policy.allowlist_active,
            "allowed_sites": policy.allowed_sites,
            "active": policy.active,
            "goal": inner.config.settings.goal,
            "language": inner.config.settings.language,
            "distraction_filters": inner.config.settings.distraction_filters,
        }))
    }

    /// The extension reports the URL of the focused tab every few seconds.
    /// Returns whether it must be blocked now (e.g. the daily allowance ran out).
    pub fn extension_activity(
        &self,
        family: &str,
        url: &str,
        seconds: u64,
        opened: bool,
    ) -> ApiResult {
        let seconds = seconds.min(60);
        let mut g = self.lock();
        let inner = &mut *g;
        inner
            .extension_seen
            .insert(family.to_string(), Instant::now());
        let (now, policy) = self.advance(inner);
        let day = now.day();
        let blocked = policy.url_blocked(url);
        let host = contentguard_core::matcher::host_of(url);
        if !host.is_empty() {
            if blocked {
                engine::record_stat(&mut inner.state, &day, &format!("site:{host}"), 0, opened);
            } else {
                let ids: Vec<String> = engine::limits_for_url(&inner.config, url)
                    .iter()
                    .map(|l| l.id.clone())
                    .collect();
                engine::record_usage(&mut inner.state, &ids, &day, seconds, opened);
                engine::record_stat(
                    &mut inner.state,
                    &day,
                    &format!("site:{host}"),
                    seconds,
                    false,
                );
            }
            inner.state_dirty = true;
        }
        let blocked_now =
            blocked || engine::effective_policy(&inner.config, &inner.state, now).url_blocked(url);
        Ok(json!({ "blocked": blocked_now }))
    }

    // ----- Licensing -----

    /// Contacts the backend to register (first run) and refresh license status.
    /// Called from the background loop; errors are cached, not surfaced.
    pub fn sync_license(&self) {
        let (ep, mut lic, first) = {
            let g = self.lock();
            (
                self.license_endpoint(&g),
                g.license.clone(),
                g.license.last_checked.is_none(),
            )
        };
        let now = Utc::now();
        let res = if first {
            license::register(&ep, &mut lic, VERSION, now)
        } else {
            license::refresh(&ep, &mut lic, now)
        };
        if res.is_err() && first {
            // No network on first run: start the trial locally so the app is usable.
            if lic.trial_started.is_none() {
                lic.trial_started = Some(now);
            }
        }
        let mut g = self.lock();
        // Preserve the install_id already in use.
        lic.install_id = g.license.install_id.clone();
        g.license = lic;
        let _ = self.save_license(&g);
    }

    pub fn refresh_license(&self) -> ApiResult {
        self.sync_license();
        let g = self.lock();
        Ok(json!({ "ok": true, "license": g.license.status(Utc::now()) }))
    }

    pub fn activate_premium(&self, code: &str) -> ApiResult {
        if code.trim().is_empty() {
            return Err(ApiError::bad_request("code"));
        }
        let (ep, mut lic) = {
            let g = self.lock();
            (self.license_endpoint(&g), g.license.clone())
        };
        license::redeem(&ep, &mut lic, code, Utc::now())
            .map_err(|e| ApiError::new(400, "activation_failed", e))?;
        let mut g = self.lock();
        lic.install_id = g.license.install_id.clone();
        g.license = lic;
        self.save_license(&g)?;
        Ok(json!({ "ok": true, "license": g.license.status(Utc::now()) }))
    }
}

/// A coarse measure of how much a config blocks. Used to let users weaken
/// their setup without a plan, while adding/strengthening needs premium.
fn block_weight(c: &Config) -> i64 {
    let s = &c.settings;
    let rules: i64 = c
        .block_lists
        .iter()
        .map(|b| (b.sites.len() + b.apps.len() + b.keywords.len()) as i64 + b.allowlist_mode as i64)
        .sum();
    let schedules = c.schedules.iter().filter(|x| x.enabled).count() as i64;
    let limits = c.limits.iter().filter(|x| x.enabled).count() as i64;
    let protection = c.protection.level as i64;
    let flags = s.require_browser_extension as i64
        + s.block_unsupported_browsers as i64
        + s.distraction_filters.len() as i64;
    rules + 5 * schedules + 5 * limits + 20 * protection + flags
}
