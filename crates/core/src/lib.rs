//! ContentGuard core: platform-independent blocking rules, schedules, limits,
//! locks and protection requests. No OS calls live here, so everything is
//! unit-testable on any machine.

pub mod challenge;
pub mod clock;
pub mod engine;
pub mod lock;
pub mod matcher;
pub mod model;
pub mod presets;
pub mod protection;
pub mod schedule;
pub mod store;

/// Random 16-hex-character identifier.
pub fn new_id() -> String {
    use rand::Rng;
    format!("{:016x}", rand::thread_rng().gen::<u64>())
}
