//! Background licensing loop: registers the install on first run and refreshes
//! the plan status periodically, so premium/trial state stays current and the
//! trial cannot be reset by wiping local data (it is anchored server-side).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::app::App;

/// Base seconds between license checks. With millions of installs every
/// request counts: 6 hours keeps the server at a few hundred requests per
/// second at 10 million devices. (`CONTENTGUARD_SYNC_SECS` shortens it for dev.)
fn base_secs() -> u64 {
    std::env::var("CONTENTGUARD_SYNC_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6 * 3600)
        .max(1)
}

/// A pseudo-random number in `0..n` (no extra dependency needed): spreads
/// devices out so they don't all call the server at the same moment.
fn jitter(n: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ d.as_secs().rotate_left(17))
        .unwrap_or(0);
    if n == 0 {
        0
    } else {
        nanos.wrapping_mul(6364136223846793005) % n
    }
}

/// Next wait: base ± 20 %.
fn next_wait() -> u64 {
    let base = base_secs();
    let spread = base / 5;
    base - spread + jitter(2 * spread + 1)
}

/// Sleeps `secs` seconds, handling "Update now" requests from the app as they
/// come; returns false when the agent is stopping.
fn wait(app: &App, stop: &AtomicBool, secs: u64) -> bool {
    for _ in 0..secs {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        if app.take_update_request() {
            app.check_update(true);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    !stop.load(Ordering::Relaxed)
}

pub fn run(app: Arc<App>, stop: Arc<AtomicBool>) {
    // A freshly installed device registers right away (its trial starts);
    // otherwise wait a little so devices that boot together spread out.
    if !app.has_checked_license() {
        app.sync_license();
        app.sync_presets_if_changed();
        app.check_update(false);
    } else if !wait(&app, &stop, 5 + jitter(115)) {
        return;
    }
    loop {
        app.sync_license();
        app.sync_presets_if_changed();
        app.check_update(false);
        let secs = if app.purchase_pending() {
            120
        } else {
            next_wait()
        };
        if !wait(&app, &stop, secs) {
            return;
        }
    }
}

/// Placeholder for telling the backend the app was uninstalled with approval.
/// The current licensing backend does not need it; kept for the uninstall flow.
pub fn report_uninstalled(_app: &App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_is_spread_around_six_hours() {
        std::env::remove_var("CONTENTGUARD_SYNC_SECS");
        for _ in 0..50 {
            let w = next_wait();
            assert!((6 * 3600 * 8 / 10..=6 * 3600 * 12 / 10).contains(&w), "{w}");
        }
        assert!(jitter(10) < 10);
        assert_eq!(jitter(0), 0);
    }
}
