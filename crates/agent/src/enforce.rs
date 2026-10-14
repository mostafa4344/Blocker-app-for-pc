//! The enforcement loop: every couple of seconds, apply the current policy to
//! the hosts file and running apps, and meter daily allowances for apps.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use focusfort_core::engine;
use focusfort_core::matcher::normalize_app;

use crate::app::App;
use crate::hosts;
use crate::paths;
use crate::processes::{browser_family, Watcher};

pub const TICK: Duration = Duration::from_secs(2);
/// A browser gets this long to connect its extension before it is closed
/// (only when "require browser extension" is on and a block is active).
const EXTENSION_GRACE_SECS: u64 = 45;
const SAVE_EVERY: Duration = Duration::from_secs(20);

pub fn run(app: Arc<App>, stop: Arc<std::sync::atomic::AtomicBool>) {
    let mut watcher = Watcher::new();
    let hosts_path = paths::hosts_file();
    let mut applied_hosts: Option<Vec<String>> = None;
    let mut closed_pids: HashSet<sysinfo::Pid> = HashSet::new();
    let mut last_save = Instant::now();
    let mut last_tick = Instant::now();

    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        let tick_secs = last_tick.elapsed().as_secs().clamp(1, 10);
        last_tick = Instant::now();

        let (now, policy, config) = {
            let mut g = app.lock();
            let (now, policy) = app.advance(&mut g);
            (now, policy, g.config.clone())
        };

        // Websites (hosts file). Rewritten only when the list changes; a
        // failed write is retried on the next tick.
        let wanted = hosts::hosts_for(&policy.blocked_sites);
        if applied_hosts.as_ref() != Some(&wanted) {
            let result = hosts::apply(&hosts_path, &wanted).map_err(|e| format!("{}: {e}", hosts_path.display()));
            if result.is_ok() {
                applied_hosts = Some(wanted);
            }
            app.lock().hosts_status = result.map(|_| ());
        }

        // Apps.
        let (procs, new_names) = watcher.scan();
        let now_epoch = now.utc.timestamp().max(0) as u64;
        let ext_seen: Vec<(String, u64)> = {
            let g = app.lock();
            g.extension_seen.iter().map(|(k, v)| (k.clone(), v.elapsed().as_secs())).collect()
        };
        let mut blocked_hits: Vec<String> = Vec::new();
        for p in &procs {
            let close_for_block = policy.app_blocked(&p.name);
            let close_for_missing_extension = config.settings.require_browser_extension
                && !policy.is_empty()
                && browser_family(&p.name).is_some_and(|f| {
                    let seen_recently = ext_seen.iter().any(|(k, ago)| k == f.key() && *ago < EXTENSION_GRACE_SECS);
                    let running_for = now_epoch.saturating_sub(p.start_time);
                    !seen_recently && running_for > EXTENSION_GRACE_SECS
                });
            if (close_for_block || close_for_missing_extension) && watcher.close(p.pid) && closed_pids.insert(p.pid) {
                blocked_hits.push(normalize_app(&p.name));
            }
        }
        closed_pids.retain(|pid| procs.iter().any(|p| &p.pid == pid));

        // Allowances for apps: count wall time while a metered app is running.
        let running: HashSet<String> = procs.iter().map(|p| normalize_app(&p.name)).collect();
        {
            let mut g = app.lock();
            let day = now.day();
            for name in &running {
                let ids: Vec<String> = engine::limits_for_app(&g.config, name).iter().map(|l| l.id.clone()).collect();
                if ids.is_empty() {
                    continue;
                }
                engine::record_usage(&mut g.state, &ids, &day, tick_secs, new_names.contains(name));
                engine::record_stat(&mut g.state, &day, &format!("app:{name}"), tick_secs, false);
                g.state_dirty = true;
            }
            blocked_hits.dedup();
            for name in &blocked_hits {
                engine::record_stat(&mut g.state, &day, &format!("app:{name}"), 0, true);
                g.state_dirty = true;
            }
            if g.state_dirty && last_save.elapsed() >= SAVE_EVERY {
                if let Err(e) = app.save_state(&mut g) {
                    eprintln!("focusfort: saving state failed: {e}");
                }
                last_save = Instant::now();
            }
        }

        std::thread::sleep(TICK);
    }

    let mut g = app.lock();
    let _ = app.save_state(&mut g);
}
