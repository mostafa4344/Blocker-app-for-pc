//! Background licensing loop: registers the install on first run and refreshes
//! the plan status periodically, so premium/trial state stays current and the
//! trial cannot be reset by wiping local data (it is anchored server-side).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::app::App;

/// Seconds between license refreshes (`CONTENTGUARD_SYNC_SECS` shortens it for dev).
fn loop_secs() -> u64 {
    std::env::var("CONTENTGUARD_SYNC_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(900)
        .max(1)
}

pub fn run(app: Arc<App>, stop: Arc<AtomicBool>) {
    // Try once promptly at startup, then on the loop interval.
    app.sync_license();
    let mut elapsed = 0u64;
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_secs(1));
        elapsed += 1;
        if elapsed >= loop_secs() {
            elapsed = 0;
            app.sync_license();
        }
    }
}

/// Placeholder for telling the backend the app was uninstalled with approval.
/// The current licensing backend does not need it; kept for the uninstall flow.
pub fn report_uninstalled(_app: &App) {}
