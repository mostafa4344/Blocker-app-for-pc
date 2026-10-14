//! A clock that cannot be wound back by changing the system time.
//!
//! The agent feeds it the wall clock plus the monotonic time elapsed since the
//! previous tick. If the two disagree (the user changed the system clock), the
//! monotonic delta is trusted instead. Time never moves backwards.

use chrono::{DateTime, Duration, Utc};

/// Wall/monotonic disagreement tolerated before the wall clock is distrusted.
const TOLERANCE_SECS: i64 = 120;

#[derive(Debug, Clone)]
pub struct TrustedClock {
    last_wall: DateTime<Utc>,
    trusted: DateTime<Utc>,
}

impl TrustedClock {
    /// `persisted` is the last trusted time saved before a restart.
    pub fn new(wall: DateTime<Utc>, persisted: Option<DateTime<Utc>>) -> Self {
        let trusted = match persisted {
            Some(p) if p > wall => p,
            _ => wall,
        };
        TrustedClock { last_wall: wall, trusted }
    }

    pub fn tick(&mut self, wall: DateTime<Utc>, monotonic_elapsed: std::time::Duration) -> DateTime<Utc> {
        let mono = Duration::from_std(monotonic_elapsed).unwrap_or(Duration::zero());
        let wall_delta = wall - self.last_wall;
        let delta = if (wall_delta - mono).num_seconds().abs() <= TOLERANCE_SECS {
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

    pub fn now(&self) -> DateTime<Utc> {
        self.trusted
    }

    /// Offset between trusted time and the wall clock; non-zero means the clock was changed.
    pub fn skew(&self) -> Duration {
        self.trusted - self.last_wall
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as Std;

    #[test]
    fn normal_progress() {
        let t0 = Utc::now();
        let mut c = TrustedClock::new(t0, None);
        assert_eq!(c.tick(t0 + Duration::seconds(5), Std::from_secs(5)), t0 + Duration::seconds(5));
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
        let now = c.tick(t0 - Duration::hours(3) + Duration::seconds(5), Std::from_secs(5));
        assert_eq!(now, t0 + Duration::seconds(10));
    }

    #[test]
    fn restart_never_goes_back() {
        let t0 = Utc::now();
        let c = TrustedClock::new(t0 - Duration::hours(1), Some(t0));
        assert_eq!(c.now(), t0);
    }
}
