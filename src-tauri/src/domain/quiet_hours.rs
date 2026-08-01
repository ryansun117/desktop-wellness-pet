use chrono::{DateTime, Days, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

use super::AppSettings;

pub fn quiet_hours_end(
    now: DateTime<Utc>,
    settings: &AppSettings,
    timezone: Tz,
) -> Option<DateTime<Utc>> {
    if !settings.quiet_hours_enabled {
        return None;
    }
    let start = NaiveTime::parse_from_str(&settings.quiet_hours_start, "%H:%M").ok()?;
    let end = NaiveTime::parse_from_str(&settings.quiet_hours_end, "%H:%M").ok()?;
    if start == end {
        return None;
    }

    let local_now = now.with_timezone(&timezone);
    let local_time = local_now.time();
    let crosses_midnight = start > end;
    let inside = if crosses_midnight {
        local_time >= start || local_time < end
    } else {
        local_time >= start && local_time < end
    };
    if !inside {
        return None;
    }

    let end_date = if crosses_midnight && local_time >= start {
        local_now.date_naive().checked_add_days(Days::new(1))?
    } else {
        local_now.date_naive()
    };
    resolve_local(timezone, end_date, end)
}

fn resolve_local(timezone: Tz, date: NaiveDate, time: NaiveTime) -> Option<DateTime<Utc>> {
    let mut candidate = NaiveDateTime::new(date, time);
    for _ in 0..=180 {
        match timezone.from_local_datetime(&candidate) {
            LocalResult::Single(value) => return Some(value.with_timezone(&Utc)),
            LocalResult::Ambiguous(first, second) => {
                return Some(first.min(second).with_timezone(&Utc));
            }
            LocalResult::None => {
                candidate = candidate.checked_add_signed(chrono::Duration::minutes(1))?
            }
        }
    }
    None
}
