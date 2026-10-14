//! Optional link to the partner/sync backend (see `backend/`).
//!
//! The agent works fully offline; the backend only adds:
//! * partner approval for protection requests,
//! * a heartbeat so the partner is told if this device goes silent
//!   while protection is on (tamper alert).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use focusfort_core::model::ProtectionLevel;
use focusfort_core::protection::RequestStatus;

use crate::app::{App, VERSION};

const LOOP_EVERY: Duration = Duration::from_secs(60);
const HEARTBEAT_EVERY_MIN: i64 = 5;

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(true)
        .build()
        .into()
}

#[derive(Deserialize)]
struct Registered {
    device_id: String,
    device_secret: String,
}

#[derive(Deserialize)]
struct RemoteRequest {
    status: String,
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

pub fn run(app: Arc<App>, stop: Arc<AtomicBool>) {
    let http = agent();
    while !stop.load(Ordering::Relaxed) {
        if let Err(e) = sync_once(&app, &http) {
            let mut g = app.lock();
            g.device.last_error = e;
            let _ = app.save_device(&g);
        }
        // Sleep in small steps so shutdown is quick.
        for _ in 0..LOOP_EVERY.as_secs() {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

pub fn sync_once(app: &App, http: &ureq::Agent) -> Result<(), String> {
    let (base, config, mut device, requests) = {
        let g = app.lock();
        (g.config.settings.backend_url.trim_end_matches('/').to_string(), g.config.clone(), g.device.clone(), g.state.requests.clone())
    };
    if base.is_empty() {
        return Ok(());
    }
    let err = |e: ureq::Error| e.to_string();
    if device.backend_url != base || device.device_id.is_empty() {
        let host = sysinfo::System::host_name().unwrap_or_else(|| "PC".into());
        let r: Registered = http
            .post(&format!("{base}/v1/devices"))
            .send_json(json!({ "name": host, "os": os_name(), "version": VERSION }))
            .map_err(err)?
            .body_mut()
            .read_json()
            .map_err(err)?;
        device = crate::app::Device { backend_url: base.clone(), device_id: r.device_id, device_secret: r.device_secret, ..Default::default() };
    }
    let auth = format!("Bearer {}", device.device_secret);
    let dev_url = format!("{base}/v1/devices/{}", device.device_id);

    // Protection settings (partner contact, level) — only when changed.
    let settings = json!({
        "protection_level": config.protection.level,
        "tamper_alerts": config.protection.tamper_alerts && config.protection.level >= ProtectionLevel::Committed,
        "partner_name": config.protection.partner.as_ref().map(|p| p.name.clone()),
        "partner_email": config.protection.partner.as_ref().map(|p| p.email.clone()),
        "language": config.settings.language,
    });
    let hash = settings.to_string();
    if hash != device.settings_hash {
        http.put(&dev_url).header("Authorization", &auth).send_json(&settings).map_err(err)?;
        device.settings_hash = hash;
    }

    let now = Utc::now();
    if device.last_heartbeat.is_none_or(|t| now - t >= chrono::Duration::minutes(HEARTBEAT_EVERY_MIN)) {
        let (active, skew) = {
            let mut g = app.lock();
            let (_, p) = app.advance(&mut g);
            (p.active.len(), g.clock.skew().num_seconds())
        };
        http.post(&format!("{dev_url}/heartbeat"))
            .header("Authorization", &auth)
            .send_json(json!({ "version": VERSION, "active_blocks": active, "clock_skew_seconds": skew }))
            .map_err(err)?;
        device.last_heartbeat = Some(now);
    }

    // Partner requests: push new ones, poll pending ones.
    let mut decisions: Vec<(String, bool)> = Vec::new();
    for r in requests.iter().filter(|r| r.needs_partner && r.status == RequestStatus::Pending) {
        if !device.pushed_requests.contains(&r.id) {
            http.post(&format!("{dev_url}/requests"))
                .header("Authorization", &auth)
                .send_json(json!({ "request_id": r.id, "kind": r.kind, "reason": r.reason, "auto_approve_at": r.available_at }))
                .map_err(err)?;
            device.pushed_requests.push(r.id.clone());
            continue;
        }
        let remote: RemoteRequest = http
            .get(&format!("{dev_url}/requests/{}", r.id))
            .header("Authorization", &auth)
            .call()
            .map_err(err)?
            .body_mut()
            .read_json()
            .map_err(err)?;
        match remote.status.as_str() {
            "approved" => decisions.push((r.id.clone(), true)),
            "denied" => decisions.push((r.id.clone(), false)),
            _ => {}
        }
    }
    device.pushed_requests.retain(|id| requests.iter().any(|r| &r.id == id && r.is_open()));
    device.last_sync_ok = Some(now);
    device.last_error.clear();

    let mut g = app.lock();
    for (id, approve) in decisions {
        if let Some(r) = g.state.requests.iter_mut().find(|r| r.id == id) {
            r.partner_decision(approve, now);
        }
    }
    g.device = device;
    app.save_device(&g).map_err(|e| e.to_string())?;
    if !g.state.requests.is_empty() {
        app.save_state(&mut g).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Tells the backend this device was uninstalled with approval, so the partner
/// gets an "uninstalled" notice instead of a tamper alert. Best effort.
pub fn report_uninstalled(app: &App) {
    let (base, device) = {
        let g = app.lock();
        (g.config.settings.backend_url.trim_end_matches('/').to_string(), g.device.clone())
    };
    if base.is_empty() || device.device_id.is_empty() {
        return;
    }
    let _ = agent()
        .post(&format!("{base}/v1/devices/{}/uninstalled", device.device_id))
        .header("Authorization", &format!("Bearer {}", device.device_secret))
        .send_json(json!({}));
}
