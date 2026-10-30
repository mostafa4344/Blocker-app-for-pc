//! Licensing: a free trial, then a paid plan. The source of truth is the
//! backend (a Supabase project via RPC, or any compatible server). Status is
//! cached locally so the app keeps working offline, but premium is only trusted
//! for a bounded time without a successful recheck, to resist offline abuse.
//!
//! Security: the plan is trusted ONLY from a license token signed by the
//! server (Ed25519, key kept in Supabase Vault; public half below). The token
//! names this Device ID and this computer's device fingerprint
//! (`devicekey.rs`), so an edited license.json, a fake server, or a license
//! copied from another PC gives no trial and no Premium. The other fields in
//! license.json are only for display.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Default free-trial length when the backend does not state one.
pub const DEFAULT_TRIAL_DAYS: i64 = 4;
/// How long a cached "premium" is trusted with no successful server recheck.
pub const OFFLINE_GRACE_DAYS: i64 = 14;
/// Before this install has ever received a signed license (first start with
/// no internet yet), setup is allowed for this long after the service starts.
pub const PROVISIONAL_MINUTES: i64 = 15;

/// Public half of the license-signing key (hex, 32 bytes Ed25519). The private
/// half never leaves the Supabase project (Vault secret `license_signing_seed`).
pub const LICENSE_PUBLIC_KEY_HEX: &str =
    "dbac7c21b268ba3d0fe41fe091d57d6fe8761c915dabdddaaf7d32ea2ec6e8b8";
/// Domain-separation prefix of the signed message (must match the SQL).
const TOKEN_PREFIX: &str = "htl-content-guard-license-v1\n";

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
    /// Last license token signed by the server (`<payload hex>.<signature hex>`).
    pub token: String,
    /// The server says this Device ID belongs to another computer.
    pub device_mismatch: bool,
    /// This computer's device fingerprint (set at start, never from the file).
    #[serde(skip)]
    pub device_key: String,
    /// Setup allowed until then while no signed license was ever received.
    #[serde(skip)]
    pub provisional_until: Option<DateTime<Utc>>,
}

/// What a valid token says (all signed by the server).
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Claims {
    pub v: u32,
    pub id: String,
    pub dk: Option<String>,
    pub plan: String,
    pub trial_started: Option<DateTime<Utc>>,
    #[serde(default)]
    pub trial_days: i64,
    pub premium_until: Option<DateTime<Utc>>,
    pub iat: DateTime<Utc>,
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || s.is_empty() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// The key tokens are checked against. Debug builds (development only) may
/// use the local mock server's key; release builds only ever trust the real one.
fn public_key_hex() -> String {
    #[cfg(debug_assertions)]
    if let Ok(k) = std::env::var("CONTENTGUARD_LICENSE_PUBKEY") {
        if !k.trim().is_empty() {
            return k.trim().to_string();
        }
    }
    LICENSE_PUBLIC_KEY_HEX.to_string()
}

/// Checks the server's signature and returns the claims, or `None`.
pub fn verify_token(token: &str, public_key_hex: &str) -> Option<Claims> {
    let (payload_hex, sig_hex) = token.trim().split_once('.')?;
    let payload = unhex(payload_hex)?;
    let sig = unhex(sig_hex)?;
    let key = unhex(public_key_hex)?;
    if key.len() != 32 || sig.len() != 64 || payload.len() > 4096 {
        return None;
    }
    let mut msg = TOKEN_PREFIX.as_bytes().to_vec();
    msg.extend_from_slice(&payload);
    if !signature_valid(&key, &msg, &sig) {
        return None;
    }
    let c: Claims = serde_json::from_slice(&payload).ok()?;
    (c.v == 1).then_some(c)
}

#[cfg(feature = "tls")]
fn signature_valid(key: &[u8], msg: &[u8], sig: &[u8]) -> bool {
    use ring::signature::{UnparsedPublicKey, ED25519};
    UnparsedPublicKey::new(&ED25519, key)
        .verify(msg, sig)
        .is_ok()
}

/// Builds without the crypto library (cross-checks only) trust no license.
#[cfg(not(feature = "tls"))]
fn signature_valid(_key: &[u8], _msg: &[u8], _sig: &[u8]) -> bool {
    false
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
            token: String::new(),
            device_mismatch: false,
            device_key: String::new(),
            provisional_until: None,
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

    /// The signed claims, if the token is genuine AND made for this Device ID
    /// on this very computer.
    pub fn claims(&self) -> Option<Claims> {
        if self.device_key.is_empty() {
            return None;
        }
        let c = verify_token(&self.token, &public_key_hex())?;
        (c.id == self.install_id && c.dk.as_deref() == Some(self.device_key.as_str())).then_some(c)
    }

    pub fn status(&self, now: DateTime<Utc>) -> Status {
        let Some(c) = self.claims() else {
            // Never verified yet: a short setup window right after start,
            // then nothing until the server has signed a license.
            if !self.device_mismatch && self.provisional_until.is_some_and(|t| now < t) {
                return Status::Trial {
                    days_left: self.trial_days.max(1),
                };
            }
            return if self.device_mismatch {
                Status::Expired
            } else {
                Status::LockedOffline
            };
        };
        let trial = trial_left(c.trial_started, c.trial_days, now);
        let paid = c.plan == "premium" && c.premium_until.is_none_or(|u| now < u);
        if paid {
            // Even lifetime premium must be re-confirmed by the server within
            // the offline grace window (the token's signed issue time).
            if now - c.iat < Duration::days(OFFLINE_GRACE_DAYS) {
                return Status::Premium;
            }
            return match trial {
                Some(days_left) => Status::Trial { days_left },
                None => Status::LockedOffline,
            };
        }
        match trial {
            Some(days_left) => Status::Trial { days_left },
            None => Status::Expired,
        }
    }

    /// Whether the user may add or strengthen blocking right now.
    pub fn allows_management(&self, now: DateTime<Utc>) -> bool {
        matches!(self.status(now), Status::Trial { .. } | Status::Premium)
    }
}

/// Days left of a trial, rounded up (a part-day still counts as a day).
fn trial_left(started: Option<DateTime<Utc>>, days: i64, now: DateTime<Utc>) -> Option<i64> {
    let end = started? + Duration::days(days.max(0));
    (now < end).then(|| {
        (end - now)
            .num_seconds()
            .saturating_add(86_399)
            .div_euclid(86_400)
    })
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
    /// Signed license token (servers since 2026-10-30).
    token: Option<String>,
    device_mismatch: Option<bool>,
    /// A refused request that isn't an HTTP error (e.g. a wrong code).
    error: Option<String>,
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
    let s = resp
        .body_mut()
        .read_json::<ServerLicense>()
        .map_err(|e| e.to_string())?;
    match s.error {
        Some(e) => Err(format!("{REJECTED}{e}")),
        None => Ok(s),
    }
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
    lic.token = s.token.unwrap_or_default();
    lic.device_mismatch = s.device_mismatch.unwrap_or(false);
    // Show what the signature vouches for, not what an unsigned answer said.
    if let Some(c) = lic.claims() {
        lic.plan = c.plan;
        lic.trial_started = c.trial_started;
        lic.trial_days = c.trial_days;
        lic.premium_until = c.premium_until;
    }
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
        "p_device_key": lic.device_key,
        "p_os": os_name(),
        "p_version": version,
    });
    match call(ep, "license_register", body) {
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
    let body = serde_json::json!({
        "p_install_id": lic.install_id,
        "p_device_key": lic.device_key,
        "p_version": version,
        "p_health": health,
    });
    match call(ep, "license_check", body) {
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
    let body = serde_json::json!({
        "p_install_id": lic.install_id,
        "p_device_key": lic.device_key,
        "p_code": code.trim(),
    });
    match call(ep, "license_redeem", body) {
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

    const ID: &str = "cg1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DK: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    #[cfg_attr(not(feature = "tls"), allow(dead_code))]
    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    /// The test signing key (same for every test, so the env var never races).
    #[cfg(feature = "tls")]
    fn test_key() -> ring::signature::Ed25519KeyPair {
        use ring::signature::KeyPair;
        let k = ring::signature::Ed25519KeyPair::from_seed_unchecked(&[7u8; 32]).unwrap();
        std::env::set_var("CONTENTGUARD_LICENSE_PUBKEY", hex(k.public_key().as_ref()));
        k
    }

    /// A token as the server would sign it.
    #[cfg(feature = "tls")]
    fn token(claims: serde_json::Value) -> String {
        let payload = claims.to_string();
        let mut msg = TOKEN_PREFIX.as_bytes().to_vec();
        msg.extend_from_slice(payload.as_bytes());
        let sig = test_key().sign(&msg);
        format!("{}.{}", hex(payload.as_bytes()), hex(sig.as_ref()))
    }

    #[cfg_attr(not(feature = "tls"), allow(dead_code))]
    fn claims(
        plan: &str,
        started: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
        iat: DateTime<Utc>,
    ) -> serde_json::Value {
        serde_json::json!({
            "v": 1, "id": ID, "dk": DK, "plan": plan,
            "trial_started": started, "trial_days": 4,
            "premium_until": until, "iat": iat,
        })
    }

    fn lic(token: String) -> License {
        License {
            install_id: ID.into(),
            device_key: DK.into(),
            token,
            ..Default::default()
        }
    }

    #[cfg(feature = "tls")]
    #[test]
    fn signed_trial_counts_down_then_expires() {
        let now = Utc::now();
        let l = lic(token(claims("trial", now, None, now)));
        assert_eq!(l.status(now), Status::Trial { days_left: 4 });
        assert!(l.allows_management(now));
        assert_eq!(
            l.status(now + Duration::days(3)),
            Status::Trial { days_left: 1 }
        );
        assert_eq!(l.status(now + Duration::days(4)), Status::Expired);
        assert!(!l.allows_management(now + Duration::days(5)));
    }

    #[cfg(feature = "tls")]
    #[test]
    fn signed_premium_needs_a_recent_check() {
        let now = Utc::now();
        let old = now - Duration::days(30);
        let l = lic(token(claims("premium", old, None, now)));
        assert_eq!(l.status(now), Status::Premium);
        // Lifetime premium, but not confirmed by the server for too long.
        assert_eq!(
            l.status(now + Duration::days(OFFLINE_GRACE_DAYS + 1)),
            Status::LockedOffline
        );
        // A subscription ends at its signed end date.
        let l = lic(token(claims(
            "premium",
            old,
            Some(now + Duration::days(10)),
            now,
        )));
        assert_eq!(l.status(now + Duration::days(9)), Status::Premium);
        assert_eq!(l.status(now + Duration::days(11)), Status::Expired);
    }

    #[cfg(feature = "tls")]
    #[test]
    fn edited_files_and_fake_servers_give_nothing() {
        let now = Utc::now();
        let old = now - Duration::days(30);
        let good = token(claims("trial", old, None, now)); // an expired trial
                                                           // Editing the readable fields of license.json changes nothing.
        let mut l = lic(good.clone());
        l.plan = "premium".into();
        l.premium_until = None;
        l.last_checked = Some(now);
        l.trial_started = Some(now);
        assert_eq!(l.status(now), Status::Expired);
        // Changing the signed payload breaks the signature.
        let (p, s) = good.split_once('.').unwrap();
        let forged_payload = hex(claims("premium", old, None, now).to_string().as_bytes());
        assert_eq!(
            lic(format!("{forged_payload}.{s}")).status(now),
            Status::LockedOffline
        );
        assert!(verify_token(&format!("{p}x.{s}"), &public_key_hex()).is_none());
        // A token signed with another key (a self-made server) is ignored.
        let other = ring::signature::Ed25519KeyPair::from_seed_unchecked(&[9u8; 32]).unwrap();
        let payload = claims("premium", old, None, now).to_string();
        let mut msg = TOKEN_PREFIX.as_bytes().to_vec();
        msg.extend_from_slice(payload.as_bytes());
        let fake = format!(
            "{}.{}",
            hex(payload.as_bytes()),
            hex(other.sign(&msg).as_ref())
        );
        test_key();
        assert_eq!(lic(fake).status(now), Status::LockedOffline);
    }

    #[cfg(feature = "tls")]
    #[test]
    fn a_license_copied_to_another_pc_is_ignored() {
        let now = Utc::now();
        let t = token(claims("premium", now, None, now));
        // Same file, different computer (different device secret).
        let mut copied = lic(t.clone());
        copied.device_key = "2".repeat(64);
        assert_eq!(copied.status(now), Status::LockedOffline);
        // Same token, different Device ID.
        let mut other_id = lic(t);
        other_id.install_id = "cg1bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        assert_eq!(other_id.status(now), Status::LockedOffline);
        // No device secret at all.
        let mut none = lic(token(claims("premium", now, None, now)));
        none.device_key.clear();
        assert!(none.claims().is_none());
    }

    #[test]
    fn short_setup_window_before_the_first_signed_license() {
        let now = Utc::now();
        let mut l = lic(String::new());
        l.provisional_until = Some(now + Duration::minutes(PROVISIONAL_MINUTES));
        assert!(matches!(l.status(now), Status::Trial { .. }));
        assert_eq!(
            l.status(now + Duration::minutes(PROVISIONAL_MINUTES + 1)),
            Status::LockedOffline
        );
        // Never for a Device ID the server says belongs to another computer.
        l.device_mismatch = true;
        assert_eq!(l.status(now), Status::Expired);
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        let key = "ab".repeat(32);
        for t in [
            "",
            ".",
            "zz.zz",
            "abcd",
            "ab.cd",
            &format!("{}.{}", "7b7d", "00".repeat(64)),
        ] {
            assert!(verify_token(t, &key).is_none(), "{t}");
        }
        assert!(unhex("abc").is_none());
        assert_eq!(unhex("0aff"), Some(vec![10, 255]));
    }
}
