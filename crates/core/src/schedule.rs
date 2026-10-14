//! Weekly schedule evaluation in local time.

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};

use crate::model::Schedule;

fn weekday_index(dt: &NaiveDateTime) -> u8 {
    dt.weekday().num_days_from_monday() as u8
}

/// Whether the schedule is active at the given local time.
/// A window that crosses midnight belongs to the day it starts on.
pub fn is_active(s: &Schedule, local: NaiveDateTime) -> bool {
    if !s.enabled || s.days.is_empty() {
        return false;
    }
    let minute = (local.hour() * 60 + local.minute()) as u16;
    let (start, end) = (s.start_minute, s.end_minute);
    if start == end {
        // Whole day.
        return s.days.contains(&weekday_index(&local));
    }
    if start < end {
        s.days.contains(&weekday_index(&local)) && minute >= start && minute < end
    } else {
        let today = s.days.contains(&weekday_index(&local)) && minute >= start;
        let yesterday = weekday_index(&(local - Duration::days(1)));
        today || (s.days.contains(&yesterday) && minute < end)
    }
}

/// Local time at which the current active window ends (for "blocked until" display).
pub fn current_window_end(s: &Schedule, local: NaiveDateTime) -> Option<NaiveDateTime> {
    if !is_active(s, local) {
        return None;
    }
    let midnight = local.date().and_hms_opt(0, 0, 0)?;
    let minute = (local.hour() * 60 + local.minute()) as u16;
    let end = if s.start_minute == s.end_minute {
        midnight + Duration::days(1)
    } else if s.start_minute < s.end_minute || minute < s.end_minute {
        midnight + Duration::minutes(s.end_minute as i64)
    } else {
        midnight + Duration::days(1) + Duration::minutes(s.end_minute as i64)
    };
    Some(end)
}

/// Total minutes per week covered by the schedule (used to compare strictness).
pub fn covered_minutes(s: &Schedule) -> Vec<(u32, u32)> {
    // Returns half-open intervals in "minutes since Monday 00:00", wrapping at the week end.
    const WEEK: u32 = 7 * 24 * 60;
    let len = if s.start_minute == s.end_minute {
        24 * 60
    } else if s.start_minute < s.end_minute {
        (s.end_minute - s.start_minute) as u32
    } else {
        (24 * 60 - s.start_minute as u32) + s.end_minute as u32
    };
    let mut out = Vec::new();
    for &d in &s.days {
        let start = d as u32 * 24 * 60 + s.start_minute as u32;
        let end = start + len;
        if end <= WEEK {
            out.push((start, end));
        } else {
            out.push((start, WEEK));
            out.push((0, end - WEEK));
        }
    }
    out
}

/// True if every minute covered by `old` is also covered by `new`.
pub fn covers(new: &Schedule, old: &Schedule) -> bool {
    let new_iv = covered_minutes(new);
    covered_minutes(old).iter().all(|&(s, e)| {
        // Walk the old interval and make sure it is fully covered by the union of new intervals.
        let mut cur = s;
        loop {
            if cur >= e {
                return true;
            }
            match new_iv
                .iter()
                .filter(|&&(ns, ne)| ns <= cur && cur < ne)
                .map(|&(_, ne)| ne)
                .max()
            {
                Some(ne) => cur = ne,
                None => return false,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }

    fn sched(days: &[u8], start: u16, end: u16) -> Schedule {
        Schedule {
            days: days.to_vec(),
            start_minute: start,
            end_minute: end,
            enabled: true,
            ..Default::default()
        }
    }

    #[test]
    fn daytime_window() {
        // 2026-10-12 is a Monday.
        let s = sched(&[0], 9 * 60, 17 * 60);
        assert!(is_active(&s, at(2026, 10, 12, 10, 0)));
        assert!(!is_active(&s, at(2026, 10, 12, 17, 0)));
        assert!(!is_active(&s, at(2026, 10, 13, 10, 0)));
    }

    #[test]
    fn overnight_window() {
        let s = sched(&[0], 22 * 60, 6 * 60);
        assert!(is_active(&s, at(2026, 10, 12, 23, 0)));
        assert!(is_active(&s, at(2026, 10, 13, 5, 59)));
        assert!(!is_active(&s, at(2026, 10, 13, 23, 0)));
        assert_eq!(
            current_window_end(&s, at(2026, 10, 12, 23, 0)),
            Some(at(2026, 10, 13, 6, 0))
        );
    }

    #[test]
    fn coverage() {
        let old = sched(&[0, 1], 9 * 60, 17 * 60);
        assert!(covers(&sched(&[0, 1, 2], 8 * 60, 18 * 60), &old));
        assert!(!covers(&sched(&[0], 8 * 60, 18 * 60), &old));
        assert!(!covers(&sched(&[0, 1], 10 * 60, 17 * 60), &old));
        let sunday_night = sched(&[6], 22 * 60, 2 * 60);
        assert!(covers(&sched(&[6, 0], 0, 0), &sunday_night));
    }
}
