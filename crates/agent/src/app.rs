//! Shared agent state and every operation the API exposes. The agent is the
//! single authority: the UI and the extension only send requests, and every
//! change is validated here against the lock rules.

use std::collections::HashMap;
use std::io;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration as StdDuration, Instant};

use chrono::{DateTime, Duration, Local, TimeZone, Timelike, Utc};
use serde_json::{json, Value};

use contentguard_core::challenge::{self, Challenge};
use contentguard_core::clock::TrustedClock;
use contentguard_core::engine::{self, Now, Policy};
use contentguard_core::lock::{self, PolicyError, UnlockProof};
use contentguard_core::model::{Config, LockKind, Protection, ProtectionLevel, State};
use contentguard_core::presets::{self, PresetDef};
use contentguard_core::protection::{Request, RequestKind, RequestStatus};
use contentguard_core::store::Store;

use crate::license::{self, License};
use crate::processes::{SUPPORTED_BROWSERS, UNSUPPORTED_BROWSERS};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_FILE: &str = "config.json";
const STATE_FILE: &str = "state.json";
const LICENSE_FILE: &str = "license.json";
/// Last ready-made list definitions received from the server.
const PRESETS_FILE: &str = "presets.json";
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
    /// Last successful time check against the server.
    pub last_time_sync: Option<Instant>,
    /// Browser family -> last time its extension reported in.
    pub extension_seen: HashMap<String, Instant>,
    pub hosts_status: Result<(), String>,
    pub state_dirty: bool,
    /// When the uninstaller last got approval (lets the service be stopped).
    pub uninstall_approved_at: Option<Instant>,
    /// Ready-made list definitions in effect (server copy or built-in).
    pub presets: Vec<PresetDef>,
    /// When presets were last fetched without a server version (old servers).
    pub presets_fetched_at: Option<Instant>,
    /// Outcome of the last automatic-update attempt, for display/health.
    pub update_status: String,
    /// The user pressed "Update now" in the app.
    pub update_requested: bool,
    /// What the updater is doing right now: "", "downloading", "installing".
    pub update_phase: &'static str,
    /// Running browsers: whether their extension is reporting in, and whether
    /// they are being closed for lacking it (shown in Settings).
    pub browser_report: Vec<Value>,
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

fn store_has(store: &Store, file: &str) -> bool {
    store.path(file).exists()
}

/// Wall-clock time for schedules and daily limits, in the pinned time zone
/// (`offset_secs` east of UTC; see `clock::TimeZoneState`).
pub fn local_at(utc: DateTime<Utc>, offset_secs: i32) -> Now {
    Now {
        utc,
        local: (utc + Duration::seconds(offset_secs as i64)).naive_utc(),
    }
}

impl App {
    pub fn open(store: Store) -> io::Result<App> {
        let mut config: Config = store.load(CONFIG_FILE)?;
        // Upgrade configs saved by older versions (only touches old files; a
        // brand-new install has no config.json and starts at the current version).
        if store_has(&store, CONFIG_FILE) && presets::migrate(&mut config) {
            store.save(CONFIG_FILE, &config)?;
        }
        let state: State = store.load(STATE_FILE)?;
        let mut license: License = store.load(LICENSE_FILE)?;
        // Anchor the install id to the machine and to an out-of-band home-dir
        // file so wiping app data (or regenerating the machine id) does not hand
        // out a fresh trial. Falls back to a random persisted id if neither is
        // available.
        license.install_id = crate::fingerprint::resolve_install_id(&license.install_id);
        license.ensure_install_id();
        // This computer's secret fingerprint: signed licenses must name it.
        license.device_key = crate::devicekey::load_or_create(store.dir()).unwrap_or_default();
        // Never verified by the server yet (first start, maybe offline): allow
        // setting up for a short while; afterwards only a signed license counts.
        if license.token.is_empty() {
            license.provisional_until =
                Some(Utc::now() + chrono::Duration::minutes(license::PROVISIONAL_MINUTES));
        }
        // Ready-made lists: the last copy from the server, else the built-ins.
        let cached: Vec<PresetDef> = store.load(PRESETS_FILE)?;
        let preset_defs = if cached.is_empty() {
            presets::builtin_defs()
        } else {
            cached
        };
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
                presets: preset_defs,
                presets_fetched_at: None,
                update_status: String::new(),
                update_requested: false,
                update_phase: "",
                clock,
                last_tick: Instant::now(),
                last_time_sync: None,
                extension_seen: HashMap::new(),
                hosts_status: Ok(()),
                state_dirty: false,
                uninstall_approved_at: None,
                browser_report: Vec::new(),
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
        // The licensing server is NOT user-configurable: pointing the agent at
        // a self-made server would fake premium. Only an environment variable
        // (set by a developer/admin for testing) can override the default.
        let _ = inner;
        license::Endpoint::resolve(
            &std::env::var("CONTENTGUARD_BACKEND_URL").unwrap_or_default(),
            &std::env::var("CONTENTGUARD_BACKEND_KEY").unwrap_or_default(),
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
        // Schedules use the pinned time zone: changing Windows' zone only
        // counts after a day while protected.
        let system_offset = Local
            .offset_from_utc_datetime(&utc.naive_utc())
            .local_minus_utc();
        let protected = lock::guarded(&inner.config)
            || !lock::locked_list_ids(&inner.config, &inner.state, utc).is_empty();
        let tz_before = inner.state.time_zone;
        let offset = inner.state.time_zone.resolve(system_offset, utc, protected);
        if inner.state.time_zone != tz_before {
            inner.state_dirty = true;
        }
        let now = local_at(utc, offset);
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
            "clock_verified": inner.clock.verified(),
            "time_zone_pending": inner.state.time_zone.pending.is_some(),
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
            "presets": inner.presets,
            "license": {
                "status": inner.license.status(now.utc),
                "install_id": inner.license.install_id,
                "plan": inner.license.plan,
                "allows_management": inner.license.allows_management(now.utc),
                "trial_days": inner.license.trial_days,
                "premium_until": inner.license.premium_until,
                "last_checked": inner.license.last_checked,
                "last_error": inner.license.last_error,
                "purchase_url": inner.license.purchase_url,
                "pay": inner.license.pay,
                "pending_purchase": inner.license.pending_purchase,
                "device_mismatch": inner.license.device_mismatch,
            },
            "update": self.update_info(inner),
            "browsers": {
                "running": inner.browser_report,
                "supported": SUPPORTED_BROWSERS,
                "unsupported": UNSUPPORTED_BROWSERS,
                "managed": {
                    "enabled": inner.config.settings.managed_extension,
                    // True once a usable extension identity is configured, so the
                    // force-install policy can actually be written.
                    "ready": crate::browserpolicy::Identity::from_settings(
                        &inner.config.settings.extension_id,
                        &inner.config.settings.extension_update_url,
                        "",
                    ).is_some(),
                },
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
        new.settings.backend_url = inner.config.settings.backend_url.clone();
        new.settings.backend_key = inner.config.settings.backend_key.clone();
        // The extension identity is not user-editable either: a bogus id would
        // silently tear down the force-installed (locked) extension.
        new.settings.extension_id = inner.config.settings.extension_id.clone();
        new.settings.extension_update_url = inner.config.settings.extension_update_url.clone();
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

    pub fn add_preset(&self, id: &str, name: Option<&str>) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let preset = inner
            .presets
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| ApiError::not_found("preset"))?;
        let now = self.now(inner);
        self.require_license(inner, now.utc)?;
        let mut list = preset.to_block_list(&inner.config.settings.language);
        if let Some(n) = name
            .map(str::trim)
            .filter(|n| !n.is_empty() && n.chars().count() <= 60)
        {
            list.name = n.to_string();
        }
        // Adding a preset that is already there tops the existing list up with
        // anything new (e.g. after an update) instead of creating a duplicate.
        let existing = inner.config.block_lists.iter_mut().find(|b| {
            b.preset == preset.id || b.name == preset.name_en || b.name == preset.name_bn
        });
        let list_id = match existing {
            Some(b) => {
                for (have, add) in [
                    (&mut b.sites, &list.sites),
                    (&mut b.apps, &list.apps),
                    (&mut b.keywords, &list.keywords),
                ] {
                    for item in add {
                        if !have.iter().any(|h| h.eq_ignore_ascii_case(item)) {
                            have.push(item.clone());
                        }
                    }
                }
                if b.preset.is_empty() {
                    b.preset = preset.id.clone();
                }
                b.always_on |= list.always_on;
                b.id.clone()
            }
            None => {
                let id = list.id.clone();
                inner.config.block_lists.push(list);
                id
            }
        };
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

    /// What the "choose what to block" picker offers: what this person used
    /// most in the last 7 days (sites and apps), programs installed on the
    /// computer, and programs running now.
    pub fn pick_suggestions(&self) -> ApiResult {
        let top = {
            let mut g = self.lock();
            let inner = &mut *g;
            let now = self.now(inner);
            let since = (now.local.date() - Duration::days(6))
                .format("%Y-%m-%d")
                .to_string();
            let mut totals: Vec<(String, u64)> = Vec::new();
            for s in inner
                .state
                .stats
                .iter()
                .filter(|s| s.day >= since && s.seconds > 0)
            {
                match totals.iter_mut().find(|(k, _)| *k == s.key) {
                    Some(t) => t.1 += s.seconds,
                    None => totals.push((s.key.clone(), s.seconds)),
                }
            }
            totals.sort_by_key(|a| std::cmp::Reverse(a.1));
            totals
                .into_iter()
                .take(12)
                .filter_map(|(k, secs)| {
                    let (kind, name) = k.split_once(':')?;
                    Some(json!({ "kind": kind, "name": name, "seconds": secs }))
                })
                .collect::<Vec<_>>()
        };
        Ok(json!({
            "top": top,
            "installed": crate::apps::installed(),
            "running": crate::apps::running(),
        }))
    }

    // ----- Sessions -----

    pub fn start_session(
        &self,
        block_list_id: &str,
        minutes: i64,
        starts_at: Option<DateTime<Utc>>,
        ends_at: Option<DateTime<Utc>>,
        lock_kind: LockKind,
    ) -> ApiResult {
        let mut g = self.lock();
        let inner = &mut *g;
        let now = self.now(inner);
        self.require_license(inner, now.utc)?;
        // Times are judged by the trusted clock, never the computer's.
        let s = match ends_at {
            Some(end) => lock::start_session_between(
                &inner.config,
                block_list_id,
                starts_at.unwrap_or(now.utc),
                end,
                lock_kind,
                now.utc,
            )?,
            None => lock::start_session(&inner.config, block_list_id, minutes, lock_kind, now.utc)?,
        };
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
        let local_minute = now.local.hour() * 60 + now.local.minute();
        lock::can_end_session(&session, proof, now.utc, local_minute)?;
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
            RequestKind::AddExceptions {
                list_id,
                sites,
                apps,
            } => {
                lock::new_exceptions(&inner.config, list_id, sites, apps)?;
                let locked =
                    lock::locked_list_ids(&inner.config, &inner.state, now.utc).contains(list_id);
                if !locked {
                    return Err(ApiError::new(
                        409,
                        "not_locked",
                        "this list is not locked: save the exceptions directly",
                    ));
                }
                // Protection off but the list is held by a locked session or a
                // locked schedule/limit: it stays as it is until that ends.
                if inner.config.protection.level == ProtectionLevel::Off {
                    return Err(ApiError::new(
                        409,
                        "list_locked_until_end",
                        "this list is locked until its session, schedule or limit ends",
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
            RequestKind::AddExceptions {
                list_id,
                sites,
                apps,
            } => {
                let mut next = inner.config.clone();
                lock::add_exceptions(&mut next, &list_id, &sites, &apps)?;
                inner.config = next;
                self.save_config(inner)?;
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

    /// Whether the OS service may be stopped now: when nothing is protected or
    /// locked, or for a while after an approved uninstall was consumed.
    pub fn service_stop_allowed(&self, inner: &Inner, now: DateTime<Utc>) -> bool {
        let recently_approved = inner
            .uninstall_approved_at
            .is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(15 * 60));
        recently_approved
            || (!lock::guarded(&inner.config)
                && lock::locked_list_ids(&inner.config, &inner.state, now).is_empty())
    }

    /// Used by the uninstaller. Returns true (and consumes the approval) when uninstalling is allowed.
    pub fn uninstall_check(&self) -> io::Result<bool> {
        let mut g = self.lock();
        let inner = &mut *g;
        self.now(inner);
        if inner.config.protection.level == ProtectionLevel::Off {
            inner.uninstall_approved_at = Some(Instant::now());
            crate::ALLOW_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
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
                inner.uninstall_approved_at = Some(Instant::now());
                crate::ALLOW_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
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
            // Only rules no exception touches are blocked instantly by the
            // extension; pages near an exception are checked with the agent
            // (`/api/ext/activity`), which knows each list's exceptions.
            "blocked_sites": policy.ext_sites,
            "keywords": policy.ext_keywords,
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
    /// Checks the time with the server and corrects the trusted clock.
    pub fn sync_time(&self) -> bool {
        let ep = {
            let g = self.lock();
            self.license_endpoint(&g)
        };
        match license::server_time(&ep) {
            Ok(t) => {
                let mut g = self.lock();
                let inner = &mut *g;
                // Bring the clock up to the moment of the answer first.
                self.advance(inner);
                inner.clock.sync_server(t);
                inner.last_time_sync = Some(Instant::now());
                inner.state_dirty = true;
                true
            }
            Err(_) => false,
        }
    }

    /// True when the time should be checked with the server: not yet since the
    /// agent started, or the computer's clock now disagrees with the trusted
    /// time (it was changed, or the PC slept) and the last check is a while ago.
    pub fn time_needs_sync(&self) -> bool {
        let g = self.lock();
        let ago = g.last_time_sync.map(|t| t.elapsed());
        !g.clock.verified()
            || (g.clock.skew().num_seconds().abs() > 120
                && ago.is_none_or(|a| a > StdDuration::from_secs(600)))
            || ago.is_none_or(|a| a > StdDuration::from_secs(6 * 3600))
    }

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
        let health = self.health_report();
        let res = if first {
            license::register(&ep, &mut lic, VERSION, now)
        } else {
            license::refresh(&ep, &mut lic, now, VERSION, health)
        };
        // A crash report has been delivered once the server accepted it.
        if res.is_ok() && lic.server_accepts_health {
            let _ = std::fs::remove_file(self.store.path(crate::CRASH_FILE));
        }
        // The server no longer knows this device (e.g. "not registered"):
        // register again. The server keys the trial on the install id, so this
        // never hands out a second trial for a device it has seen.
        if !first
            && res
                .as_ref()
                .is_err_and(|e| e.starts_with(license::REJECTED))
        {
            let _ = license::register(&ep, &mut lic, VERSION, now);
        }
        let mut g = self.lock();
        // Preserve the install_id already in use.
        lic.install_id = g.license.install_id.clone();
        g.license = lic;
        let _ = self.save_license(&g);
    }

    /// Fetches the admin-managed ready-made lists from the server and brings
    /// the user's lists made from them up to date (see `presets::apply_update`).
    /// Keeps the current definitions when offline or when the answer looks wrong.
    pub fn sync_presets(&self) -> bool {
        let ep = {
            let g = self.lock();
            self.license_endpoint(&g)
        };
        let Ok(remote) = license::fetch_presets(&ep) else {
            return false;
        };
        let fresh: Vec<PresetDef> = remote
            .into_iter()
            .filter_map(PresetDef::sanitized)
            .collect();
        if fresh.is_empty() {
            return true; // empty table or nothing usable: keep what we have
        }
        let builtin = presets::builtin_defs();
        let mut g = self.lock();
        let inner = &mut *g;
        let mut effective: Vec<PresetDef> = Vec::new();
        let mut config_changed = false;
        for new in fresh {
            let prev = inner
                .presets
                .iter()
                .chain(builtin.iter())
                .find(|p| p.id == new.id)
                .cloned()
                .unwrap_or_else(|| PresetDef {
                    id: new.id.clone(),
                    ..Default::default()
                });
            if !presets::plausible_update(&prev, &new) {
                eprintln!(
                    "contentguard: ignoring implausible update of preset {}",
                    new.id
                );
                effective.push(prev);
                continue;
            }
            config_changed |= presets::apply_update(&mut inner.config, &prev, &new);
            effective.push(new);
        }
        if config_changed {
            let _ = self.save_config(inner);
        }
        if effective != inner.presets {
            inner.presets = effective;
            let _ = self.store.save(PRESETS_FILE, &inner.presets);
        }
        true
    }

    pub fn refresh_license(&self) -> ApiResult {
        // Checked right away (the user just bought, or pressed "refresh"), but
        // never more than once a minute per device.
        let recent = self
            .lock()
            .license
            .last_checked
            .is_some_and(|t| Utc::now() - t < Duration::seconds(60));
        if !recent {
            self.sync_license();
            self.sync_presets_if_changed();
        }
        let g = self.lock();
        Ok(json!({ "ok": true, "license": g.license.status(Utc::now()) }))
    }

    /// Update information for the app screen: the current version and, when a
    /// genuine newer release exists, its version and "what's new" text.
    fn update_info(&self, inner: &Inner) -> Value {
        #[cfg(feature = "tls")]
        let available = inner
            .license
            .latest_release
            .clone()
            .and_then(|v| serde_json::from_value::<crate::update::Release>(v).ok())
            .filter(|r| crate::update::available(r, VERSION))
            .map(|r| json!({ "version": r.version, "notes": r.notes, "notes_bn": r.notes_bn }));
        #[cfg(not(feature = "tls"))]
        let available: Option<Value> = None;
        json!({
            "current": VERSION,
            "available": available,
            "phase": inner.update_phase,
            "requested": inner.update_requested,
            "status": inner.update_status,
        })
    }

    /// "Update now" from the app: the background updater picks it up within a
    /// second (it installs even if this device isn't in the rollout yet).
    pub fn request_update(&self) -> ApiResult {
        let mut g = self.lock();
        let has = self.update_info(&g)["available"].is_object();
        if !has {
            return Err(ApiError::new(409, "no_update", "no update available"));
        }
        g.update_requested = true;
        Ok(json!({ "ok": true }))
    }

    /// Consumes a pending "Update now" request.
    pub fn take_update_request(&self) -> bool {
        std::mem::take(&mut self.lock().update_requested)
    }

    /// Installs a newer signed release announced by the server: automatically
    /// when this device is in the rollout, or right away when the user asked
    /// (`by_user`). Settings are kept: the installer upgrades in place.
    #[cfg(feature = "tls")]
    pub fn check_update(&self, by_user: bool) {
        use crate::update;
        let (release, install_id, tried_version, attempts) = {
            let g = self.lock();
            let r: Option<update::Release> = g
                .license
                .latest_release
                .clone()
                .and_then(|v| serde_json::from_value(v).ok());
            (
                r,
                g.license.install_id.clone(),
                g.license.update_attempt_version.clone(),
                g.license.update_attempts,
            )
        };
        let Some(r) = release else { return };
        let go = if by_user {
            update::available(&r, VERSION)
        } else {
            update::wanted(&r, VERSION, &install_id)
        };
        if !go {
            return;
        }
        let attempts = if tried_version == r.version {
            attempts
        } else {
            0
        };
        // Automatic tries stop after a few failures; the user may always retry.
        if attempts >= update::MAX_ATTEMPTS && !by_user {
            return;
        }
        self.lock().update_phase = "downloading";
        {
            let mut g = self.lock();
            g.license.update_attempt_version = r.version.clone();
            g.license.update_attempts = attempts + 1;
            let _ = self.save_license(&g);
        }
        let result = update::download_and_install(&r);
        let mut g = self.lock();
        // "installing": the installer now restarts the service on the new version.
        g.update_phase = match &result {
            Ok(s) if !s.contains("dry run") => "installing",
            _ => "",
        };
        g.update_status = match result {
            Ok(s) => format!("{}: {s}", r.version),
            Err(e) => format!("{}: failed: {e}", r.version),
        };
        eprintln!("contentguard: update {}", g.update_status);
    }

    #[cfg(not(feature = "tls"))]
    pub fn check_update(&self, _by_user: bool) {}

    /// Anonymous health summary sent with license checks (only to servers that
    /// accept it). No browsing data, no app names, no personal data: just
    /// whether the parts of the app work, so problems can be found and fixed.
    pub fn health_report(&self) -> Value {
        let crash: String = std::fs::read_to_string(self.store.path(crate::CRASH_FILE))
            .unwrap_or_default()
            .chars()
            .take(400)
            .collect();
        let g = self.lock();
        let ext_browsers = g
            .extension_seen
            .values()
            .filter(|t| t.elapsed().as_secs() < 3600)
            .count();
        let hosts_error = match &g.hosts_status {
            Ok(()) => String::new(),
            Err(e) if e.contains("denied") || e.contains("os error 5") => "permission".into(),
            Err(_) => "other".into(),
        };
        let mut update: String = g.update_status.chars().take(120).collect();
        if update.is_empty() {
            update = "none".into();
        }
        json!({
            "os": sysinfo::System::long_os_version().unwrap_or_default(),
            "hosts_ok": g.hosts_status.is_ok(),
            "hosts_error": hosts_error,
            "protection": g.config.protection.level,
            "always_on_lists": g.config.block_lists.iter().filter(|b| b.always_on).count(),
            "extension_browsers": ext_browsers,
            "update": update,
            "crash": crash,
        })
    }

    /// A purchase from the app's Premium page: the customer paid with
    /// bKash/Nagad/Rocket and typed the TrxID. The admin approves it in
    /// Supabase (purchase_requests -> status = approved) and Premium turns on.
    pub fn submit_purchase(
        &self,
        plan: &str,
        method: &str,
        txn_id: &str,
        contact: &str,
    ) -> ApiResult {
        let txn: String = txn_id.trim().to_ascii_uppercase();
        // Plans come from the server's "plans" table: the classic codes, or
        // "<months>m" (e.g. "2m"). The server checks the plan really exists.
        let months_code = plan.strip_suffix('m').is_some_and(|n| {
            !n.is_empty() && n.len() <= 3 && n.bytes().all(|b| b.is_ascii_digit())
        });
        if !(matches!(plan, "monthly" | "yearly" | "lifetime") || months_code)
            || !matches!(method, "bkash" | "nagad" | "rocket" | "upay")
        {
            return Err(ApiError::bad_request("plan/method"));
        }
        if !(4..=40).contains(&txn.len()) || !txn.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ApiError::new(400, "invalid_txn", "TrxID looks wrong"));
        }
        let contact: String = contact.trim().chars().take(40).collect();
        let (ep, install_id) = {
            let g = self.lock();
            (self.license_endpoint(&g), g.license.install_id.clone())
        };
        license::submit_purchase(&ep, &install_id, plan, method, &txn, &contact).map_err(|e| {
            if e.contains("already used") {
                ApiError::new(409, "txn_used", e)
            } else if e.contains("too many") {
                ApiError::new(429, "too_many_pending", e)
            } else if e.starts_with(license::REJECTED) {
                ApiError::new(400, "invalid_txn", e)
            } else {
                ApiError::new(503, "license_offline", e)
            }
        })?;
        let mut g = self.lock();
        g.license.pending_purchase = Some(json!({
            "plan": plan, "method": method, "txn_id": txn, "at": Utc::now(),
        }));
        self.save_license(&g)?;
        Ok(json!({ "ok": true }))
    }

    /// While a purchase waits for approval the app checks every couple of
    /// minutes (for two days), so Premium appears soon after the admin approves.
    pub fn purchase_pending(&self) -> bool {
        let mut g = self.lock();
        let Some(p) = g.license.pending_purchase.clone() else {
            return false;
        };
        let premium = matches!(g.license.status(Utc::now()), license::Status::Premium);
        let old = p["at"]
            .as_str()
            .and_then(|t| t.parse::<DateTime<Utc>>().ok())
            .is_none_or(|t| Utc::now() - t > Duration::days(2));
        if premium || old {
            g.license.pending_purchase = None;
            let _ = self.save_license(&g);
            return false;
        }
        true
    }

    /// Whether this device has ever reached the license server.
    pub fn has_checked_license(&self) -> bool {
        self.lock().license.last_checked.is_some()
    }

    /// Downloads the preset lists only when the admin changed them: the
    /// license answer carries a small version string, so millions of devices
    /// don't re-download unchanged lists. Servers without that field fall back
    /// to one download per day.
    pub fn sync_presets_if_changed(&self) {
        let (server_version, fetched, fetched_at) = {
            let g = self.lock();
            (
                g.license.presets_version.clone(),
                g.license.presets_fetched.clone(),
                g.presets_fetched_at,
            )
        };
        let due = if server_version.is_empty() {
            fetched_at.is_none_or(|t| t.elapsed() >= std::time::Duration::from_secs(24 * 3600))
        } else {
            server_version != fetched
        };
        if !due {
            return;
        }
        if self.sync_presets() {
            let mut g = self.lock();
            g.presets_fetched_at = Some(Instant::now());
            g.license.presets_fetched = server_version;
            let _ = self.save_license(&g);
        }
    }

    pub fn activate_premium(&self, code: &str) -> ApiResult {
        if code.trim().is_empty() {
            return Err(ApiError::bad_request("code"));
        }
        let (ep, mut lic) = {
            let g = self.lock();
            (self.license_endpoint(&g), g.license.clone())
        };
        license::redeem(&ep, &mut lic, code, Utc::now()).map_err(|e| {
            if e.starts_with(license::REJECTED) {
                ApiError::new(400, "invalid_code", e.trim_start_matches(license::REJECTED))
            } else {
                ApiError::new(503, "license_offline", e)
            }
        })?;
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
        .map(|b| {
            (b.sites.len() + b.apps.len() + b.keywords.len()) as i64
                + b.allowlist_mode as i64
                + 3 * b.always_on as i64
        })
        .sum();
    let schedules = c.schedules.iter().filter(|x| x.enabled).count() as i64;
    let limits = c.limits.iter().filter(|x| x.enabled).count() as i64;
    let protection = c.protection.level as i64;
    let flags = s.require_browser_extension as i64
        + s.block_unsupported_browsers as i64
        + s.force_safe_search as i64
        + s.block_private_windows as i64
        + (s.dns_filter != "off") as i64
        + s.distraction_filters.len() as i64;
    rules + 5 * schedules + 5 * limits + 20 * protection + flags
}
