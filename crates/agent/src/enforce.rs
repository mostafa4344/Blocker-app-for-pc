//! The enforcement loop: every couple of seconds, apply the current policy to
//! the hosts file and running apps, and meter daily allowances for apps.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use contentguard_core::engine;
use contentguard_core::matcher::normalize_app;

use crate::app::App;
use crate::hosts;
use crate::paths;
use crate::processes::{
    browser_brand, browser_family, browser_support, report_covers, BrowserFamily, BrowserSupport,
    Watcher,
};

/// Blocked host names, plus SafeSearch `(ip, host)` redirects, last written.
type AppliedHosts = (Vec<String>, Vec<(String, String)>);

pub const TICK: Duration = Duration::from_secs(2);
/// A browser gets this long to connect its extension before it is closed
/// (only when "require browser extension" is on and a block is active).
const EXTENSION_GRACE_SECS: u64 = 90;
const SAVE_EVERY: Duration = Duration::from_secs(20);
/// How often the managed-browser extension policy is re-asserted so a deleted
/// policy key is restored quickly.
const POLICY_REASSERT: Duration = Duration::from_secs(15);
/// How often the hosts file is re-read to undo manual edits.
const HOSTS_RECHECK: Duration = Duration::from_secs(6);

pub fn run(app: Arc<App>, stop: Arc<std::sync::atomic::AtomicBool>) {
    let mut watcher = Watcher::new();
    let hosts_path = paths::hosts_file();
    let mut safe_search = crate::safesearch::SafeSearch::new();
    let mut applied_hosts: Option<AppliedHosts> = None;
    // The browser-policy identity currently asserted (None = policy removed).
    let mut applied_policy: Option<Option<crate::browserpolicy::Identity>> = None;
    let mut last_policy_reassert = Instant::now();
    let mut last_hosts_check = Instant::now();
    let mut cached_sites: Vec<String> = Vec::new();
    let mut cached_hosts: Vec<String> = Vec::new();
    let mut applied_private: Option<bool> = None;
    let mut last_private_check = Instant::now();
    let mut closed_pids: HashSet<sysinfo::Pid> = HashSet::new();
    let mut last_save = Instant::now();
    let mut last_tick = Instant::now();

    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        let tick_secs = last_tick.elapsed().as_secs().clamp(1, 10);
        last_tick = Instant::now();

        let (now, policy, config) = {
            let mut g = app.lock();
            let (now, policy) = app.advance(&mut g);
            let allow_stop = app.service_stop_allowed(&g, now.utc);
            crate::ALLOW_STOP.store(allow_stop, std::sync::atomic::Ordering::Relaxed);
            (now, policy, g.config.clone())
        };

        // Websites (hosts file) + SafeSearch redirects. Rewritten only when the
        // desired entries change; a failed write is retried on the next tick.
        // Recomputed only when the blocked sites change (big lists are costly).
        if cached_sites != policy.blocked_sites {
            cached_hosts = hosts::hosts_for(&policy.blocked_sites);
            cached_sites = policy.blocked_sites.clone();
        }
        let wanted = cached_hosts.clone();
        let safe = safe_search
            .entries(config.settings.force_safe_search, &wanted)
            .to_vec();
        // Also re-checked every few seconds so hand-edits to the hosts file
        // (deleting our section) are undone quickly. `apply` only writes when
        // the file content actually differs.
        let hosts_recheck_due = last_hosts_check.elapsed() >= HOSTS_RECHECK;
        if hosts_recheck_due || applied_hosts.as_ref() != Some(&(wanted.clone(), safe.clone())) {
            last_hosts_check = Instant::now();
            let result = hosts::apply(&hosts_path, &wanted, &safe)
                .map_err(|e| format!("{}: {e}", hosts_path.display()));
            if result.is_ok() {
                applied_hosts = Some((wanted, safe));
            }
            app.lock().hosts_status = result.map(|_| ());
        }

        // Managed-browser policy: keep the force-installed extension asserted so
        // the user cannot remove it, or tear it down when the feature is off.
        // `apply` is also re-run periodically so a deleted policy key is restored.
        {
            let s = &config.settings;
            let wanted: Option<crate::browserpolicy::Identity> = if s.managed_extension {
                crate::browserpolicy::Identity::from_settings(
                    &s.extension_id,
                    &s.extension_update_url,
                    "",
                )
            } else {
                None
            };
            let changed = applied_policy.as_ref() != Some(&wanted);
            // Re-assert periodically so a deleted policy key is restored within
            // seconds, without spawning `reg` processes on every 2 s tick.
            let reassert_due = last_policy_reassert.elapsed() >= POLICY_REASSERT;
            match &wanted {
                Some(id) if changed || reassert_due => {
                    crate::browserpolicy::apply(id);
                    last_policy_reassert = Instant::now();
                }
                Some(_) => {}
                None if changed => crate::browserpolicy::remove(),
                None => {}
            }
            applied_policy = Some(wanted);

            // Private / incognito windows: asserted on change and re-checked
            // with the same cadence as the extension policy.
            let private = s.block_private_windows;
            let private_due = last_private_check.elapsed() >= POLICY_REASSERT;
            if applied_private != Some(private) || (private && private_due) {
                crate::browserpolicy::set_private_windows_blocked(private);
                applied_private = Some(private);
                last_private_check = Instant::now();
            }
        }

        // Apps.
        let (procs, new_names) = watcher.scan();
        let now_epoch = now.utc.timestamp().max(0) as u64;
        let ext_seen: Vec<(String, u64)> = {
            let g = app.lock();
            g.extension_seen
                .iter()
                .map(|(k, v)| (k.clone(), v.elapsed().as_secs()))
                .collect()
        };
        let blocking = !policy.is_empty();
        let mut blocked_hits: Vec<String> = Vec::new();
        // A supported browser whose extension isn't reporting in (not installed,
        // disabled, or filtering not working) is closed while blocking when
        // "require extension" is on. This is separate from closing browsers that
        // can't be filtered at all (Tor...), which is safe to enable for everyone:
        // the extension can't be force-installed in every browser yet (Edge on
        // home PCs only takes Edge Add-ons extensions, Firefox needs AMO).
        let guard_browsers = config.settings.require_browser_extension;
        let mut report: Vec<(&'static str, bool, bool)> = Vec::new();
        // Distinct browser brands running per engine family, to know whether an
        // old family-only extension report can be attributed to one browser.
        let mut brands_running: Vec<(BrowserFamily, &'static str)> = Vec::new();
        for p in &procs {
            if let (Some((brand, _)), Some(family)) =
                (browser_brand(&p.name), browser_family(&p.name))
            {
                if !brands_running.contains(&(family, brand)) {
                    brands_running.push((family, brand));
                }
            }
        }
        for p in &procs {
            let close_for_block = policy.app_blocked(&p.name);
            let mut close_for_missing_extension = false;
            if let (Some((brand, display)), Some(family)) =
                (browser_brand(&p.name), browser_family(&p.name))
            {
                let sole = brands_running.iter().filter(|(f, _)| *f == family).count() <= 1;
                let protected = ext_seen.iter().any(|(k, ago)| {
                    *ago < EXTENSION_GRACE_SECS && report_covers(k, brand, family, sole)
                });
                let running_for = now_epoch.saturating_sub(p.start_time);
                close_for_missing_extension =
                    guard_browsers && blocking && !protected && running_for > EXTENSION_GRACE_SECS;
                let will_close = guard_browsers && blocking && !protected;
                if !report.iter().any(|(d, _, _)| *d == display) {
                    report.push((display, protected, will_close));
                }
            }
            // Browsers we cannot scan are a bypass route: close them while blocking.
            let close_for_unsupported = config.settings.block_unsupported_browsers
                && blocking
                && browser_support(&p.name) == BrowserSupport::Unsupported;
            if (close_for_block || close_for_missing_extension || close_for_unsupported)
                && watcher.close(p.pid)
                && closed_pids.insert(p.pid)
            {
                blocked_hits.push(normalize_app(&p.name));
            }
        }
        closed_pids.retain(|pid| procs.iter().any(|p| &p.pid == pid));
        app.lock().browser_report = report
            .into_iter()
            .map(|(name, protected, closes)| {
                serde_json::json!({ "name": name, "protected": protected, "closes": closes })
            })
            .collect();

        // Allowances for apps: count wall time while a metered app is running.
        let running: HashSet<String> = procs.iter().map(|p| normalize_app(&p.name)).collect();
        {
            let mut g = app.lock();
            let day = now.day();
            for name in &running {
                let ids: Vec<String> = engine::limits_for_app(&g.config, name)
                    .iter()
                    .map(|l| l.id.clone())
                    .collect();
                if ids.is_empty() {
                    continue;
                }
                engine::record_usage(
                    &mut g.state,
                    &ids,
                    &day,
                    tick_secs,
                    new_names.contains(name),
                );
                engine::record_stat(&mut g.state, &day, &format!("app:{name}"), tick_secs, false);
                g.state_dirty = true;
            }
            blocked_hits.dedup();
            for name in &blocked_hits {
                engine::record_stat(&mut g.state, &day, &format!("app:{name}"), 0, true);
                g.state_dirty = true;
            }
            // Also save at least every minute: after a restart the clock resumes
            // from the last saved trusted time until the server confirms it.
            if (g.state_dirty && last_save.elapsed() >= SAVE_EVERY)
                || last_save.elapsed() >= Duration::from_secs(60)
            {
                if let Err(e) = app.save_state(&mut g) {
                    eprintln!("contentguard: saving state failed: {e}");
                }
                last_save = Instant::now();
            }
        }

        std::thread::sleep(TICK);
    }

    let mut g = app.lock();
    let _ = app.save_state(&mut g);
}
