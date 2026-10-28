//! A clock that cannot be moved by changing the system time.
//!
//! Locks, waiting times and daily limits all depend on "now", and the people
//! this app protects will try changing the computer's clock. So:
//!
//! * While running, the wall clock only counts when it agrees with the
//!   monotonic clock (which the user can't change). A jump - or many small
//!   nudges - is replaced by the monotonic time actually elapsed.
//! * After a restart, the wall clock isn't believed either (it could have been
//!   moved while the agent was off): time continues from the last trusted
//!   moment until the server confirms the real time.
//! * The server's time (from its HTTPS answer, which can't be faked) is the
//!   real authority: it sets the clock forward or back.
//!
//! Without internet after a restart the clock lags real time (it ignores the
//! time the computer was off). That errs on the strict side: waits and locks
//! last longer, never shorter, until the device is online again.

use chrono::{DateTime, Duration, Utc};

/// Wall/monotonic disagreement always tolerated (small clock corrections).
const BASE_TOLERANCE_SECS: i64 = 5;
/// Plus this fraction of the elapsed time (normal clock drift), in 1/1000.
const DRIFT_PER_MILLE: i64 = 10;

#[derive(Debug, Clone)]
pub struct TrustedClock {
    last_wall: DateTime<Utc>,
    trusted: DateTime<Utc>,
    verified: bool,
}

impl TrustedClock {
    /// `persisted` is the last trusted time saved before a restart. The wall
    /// clock is only used on the very first start (nothing persisted yet).
    pub fn new(wall: DateTime<Utc>, persisted: Option<DateTime<Utc>>) -> Self {
        TrustedClock {
            last_wall: wall,
            trusted: persisted.unwrap_or(wall),
            verified: false,
        }
    }

    pub fn tick(
        &mut self,
        wall: DateTime<Utc>,
        monotonic_elapsed: std::time::Duration,
    ) -> DateTime<Utc> {
        let mono = Duration::from_std(monotonic_elapsed).unwrap_or(Duration::zero());
        let wall_delta = wall - self.last_wall;
        let tolerance = BASE_TOLERANCE_SECS + mono.num_seconds() * DRIFT_PER_MILLE / 1000;
        let delta = if (wall_delta - mono).num_seconds().abs() <= tolerance {
            wall_delta
        } else {
            mono
        };
        if delta > Duration::zero() {
            self.trusted += delta;
        }
        self.last_wall = wall;
        self.trusted
    }

    /// Sets the clock to the server's time (`server` already corrected for the
    /// network delay). The server is authoritative in both directions.
    pub fn sync_server(&mut self, server: DateTime<Utc>) {
        self.trusted = server;
        self.verified = true;
    }

    /// True once the server's time has been applied since the agent started.
    pub fn verified(&self) -> bool {
        self.verified
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.trusted
    }

    /// Offset between trusted time and the wall clock; non-zero means the clock was changed.
    pub fn skew(&self) -> Duration {
        self.trusted - self.last_wall
    }
}

/// How long a new time zone must stay set before schedules and daily limits
/// follow it while protection is on.
pub const TIMEZONE_DELAY_HOURS: i64 = 24;

/// The time zone (UTC offset, seconds) that schedules and daily limits use.
/// Anyone can change Windows' time zone without admin rights, which would
/// shift "9:00-17:00" to other hours; so while protected a new zone is only
/// adopted after it has been in place for [`TIMEZONE_DELAY_HOURS`] (real
/// travel and daylight saving still work, a day later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct TimeZoneState {
    /// Offset in use. `None` until first seen.
    pub offset: Option<i32>,
    /// A different system offset waiting to be adopted, and since when.
    pub pending: Option<(i32, DateTime<Utc>)>,
}

impl TimeZoneState {
    /// Updates for the current system offset; returns the offset to use.
    pub fn resolve(&mut self, system: i32, now: DateTime<Utc>, protected: bool) -> i32 {
        let current = match self.offset {
            None => {
                self.offset = Some(system);
                return system;
            }
            Some(o) => o,
        };
        if system == current {
            self.pending = None;
            return current;
        }
        if !protected {
            self.offset = Some(system);
            self.pending = None;
            return system;
        }
        match self.pending {
            Some((o, since)) if o == system => {
                if now - since >= Duration::hours(TIMEZONE_DELAY_HOURS) {
                    self.offset = Some(system);
                    self.pending = None;
                    return system;
                }
            }
            _ => self.pending = Some((system, now)),
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as Std;

    #[test]
    fn time_zone_change_waits_a_day_while_protected() {
        let t0 = Utc::now();
        let dhaka = 6 * 3600;
        let tokyo = 9 * 3600;
        let mut tz = TimeZoneState::default();
        assert_eq!(tz.resolve(dhaka, t0, true), dhaka);
        // Switching zones to dodge a schedule doesn't work...
        assert_eq!(tz.resolve(tokyo, t0 + Duration::minutes(1), true), dhaka);
        assert_eq!(tz.resolve(tokyo, t0 + Duration::hours(23), true), dhaka);
        // ...switching back and forth restarts the wait...
        assert_eq!(tz.resolve(dhaka, t0 + Duration::hours(23), true), dhaka);
        assert_eq!(tz.resolve(tokyo, t0 + Duration::hours(24), true), dhaka);
        // ...and a zone kept for a full day is adopted (travel, daylight saving).
        assert_eq!(tz.resolve(tokyo, t0 + Duration::hours(48), true), tokyo);
        // Without protection a change applies at once.
        assert_eq!(tz.resolve(dhaka, t0 + Duration::hours(49), false), dhaka);
    }

    #[test]
    fn normal_progress() {
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0, None);
        assert_eq!(
            c.tick(t0 + Duration::seconds(5), Std::from_secs(5)),
            t0 + Duration::seconds(5)
        );
    }

    #[test]
    fn clock_set_forward_is_ignored() {
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0, None);
        let now = c.tick(t0 + Duration::days(1), Std::from_secs(5));
        assert_eq!(now, t0 + Duration::seconds(5));
    }

    #[test]
    fn clock_set_back_is_ignored() {
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0, None);
        let now = c.tick(t0 - Duration::hours(3), Std::from_secs(5));
        assert_eq!(now, t0 + Duration::seconds(5));
        // Subsequent normal ticks keep advancing from the trusted time.
        let now = c.tick(
            t0 - Duration::hours(3) + Duration::seconds(5),
            Std::from_secs(5),
        );
        assert_eq!(now, t0 + Duration::seconds(10));
    }

    #[test]
    fn small_nudges_every_tick_do_not_add_up() {
        // A script moving the clock 2 minutes forward every 2 seconds used to
        // pass the old 120-second tolerance and run time 60x faster.
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0, None);
        let mut wall = t0;
        for _ in 0..1800 {
            wall += Duration::seconds(2 + 118);
            c.tick(wall, Std::from_secs(2));
        }
        assert_eq!(c.now(), t0 + Duration::seconds(3600));
        // A few seconds of normal correction are still accepted.
        let before = c.now();
        c.tick(wall + Duration::seconds(4), Std::from_secs(2));
        assert_eq!(c.now(), before + Duration::seconds(4));
    }

    #[test]
    fn restart_never_goes_back() {
        let t0 = Utc::now();
        let c = TrustedClock::new(t0 - Duration::hours(1), Some(t0));
        assert_eq!(c.now(), t0);
    }

    #[test]
    fn clock_moved_forward_while_off_is_not_believed() {
        // Set the clock 3 days ahead, restart: still the last trusted moment...
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0 + Duration::days(3), Some(t0));
        assert_eq!(c.now(), t0);
        assert!(!c.verified());
        let now = c.tick(
            t0 + Duration::days(3) + Duration::seconds(2),
            Std::from_secs(2),
        );
        assert_eq!(now, t0 + Duration::seconds(2));
        // ...until the server says what time it really is.
        c.sync_server(t0 + Duration::hours(8));
        assert!(c.verified());
        assert_eq!(c.now(), t0 + Duration::hours(8));
        assert_eq!(
            c.tick(
                t0 + Duration::days(3) + Duration::seconds(4),
                Std::from_secs(2)
            ),
            t0 + Duration::hours(8) + Duration::seconds(2)
        );
    }
}
