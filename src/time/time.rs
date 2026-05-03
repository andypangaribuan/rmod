/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use chrono::Duration as ChronoDuration;
use chrono::SecondsFormat;
pub use chrono::{self, DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
pub use tokio::time::Duration;

pub fn to_rfc3339(dt: DateTime<Utc>) -> String {
    if let Some(tz) = crate::store::get_timezone().and_then(|tz_str| tz_str.parse::<Tz>().ok()) {
        return dt.with_timezone(&tz).to_rfc3339_opts(SecondsFormat::Secs, false);
    }

    dt.to_rfc3339_opts(SecondsFormat::Secs, false)
}

pub fn to_rfc3339_full(dt: DateTime<Utc>) -> String {
    if let Some(tz) = crate::store::get_timezone().and_then(|tz_str| tz_str.parse::<Tz>().ok()) {
        return dt.with_timezone(&tz).to_rfc3339();
    }

    dt.to_rfc3339()
}

pub fn format(dt: DateTime<Utc>, format: &str) -> String {
    if let Some(tz) = crate::store::get_timezone().and_then(|tz_str| tz_str.parse::<Tz>().ok()) {
        return dt.with_timezone(&tz).format(format).to_string();
    }

    dt.format(format).to_string()
}

pub fn from_rfc3339(rfc3339: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(rfc3339).map(|dt| dt.with_timezone(&Utc))
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_tz() -> DateTime<Tz> {
    let tz = crate::store::get_timezone().and_then(|tz_str| tz_str.parse::<Tz>().ok()).unwrap_or(Tz::UTC);
    Utc::now().with_timezone(&tz)
}

pub fn is_timezone_utc() -> bool {
    let tz = crate::store::get_timezone().unwrap_or_default();
    tz.is_empty() || tz.eq_ignore_ascii_case("utc")
}

pub trait ToDuration {
    fn to_duration(&self) -> Duration;
}

impl ToDuration for Duration {
    fn to_duration(&self) -> Duration {
        *self
    }
}

impl ToDuration for &str {
    fn to_duration(&self) -> Duration {
        to_duration(self)
    }
}

impl ToDuration for String {
    fn to_duration(&self) -> Duration {
        to_duration(self)
    }
}

pub async fn sleep<T: ToDuration>(duration: T) {
    tokio::time::sleep(duration.to_duration()).await;
}

pub fn to_duration(duration: &str) -> Duration {
    let delta = to_delta(duration);
    let ms = delta.num_milliseconds();
    if ms < 0 { Duration::from_secs(0) } else { Duration::from_millis(ms as u64) }
}

pub fn to_delta(duration: &str) -> TimeDelta {
    if duration.is_empty() {
        return TimeDelta::zero();
    }

    let (duration, is_neg) =
        if let Some(stripped) = duration.trim().strip_prefix('-') { (stripped, true) } else { (duration.trim(), false) };

    let mut val_str = duration;
    let mut unit = "s";

    // Longest to shortest to avoid partial matches on trailing 's'
    let suffixes = [
        ("ms", "milliseconds"),
        ("ms", "millisecond"),
        ("s", "seconds"),
        ("s", "second"),
        ("m", "minutes"),
        ("m", "minute"),
        ("h", "hours"),
        ("h", "hour"),
        ("d", "days"),
        ("d", "day"),
        ("ms", "ms"),
        ("s", "s"),
        ("m", "m"),
        ("h", "h"),
        ("d", "d"),
    ];

    for (u, suf) in suffixes {
        if let Some(stripped) = duration.strip_suffix(suf) {
            val_str = stripped.trim_end();
            unit = u;
            break;
        }
    }

    let val = val_str.parse::<i64>().unwrap_or(0);

    let delta = match unit {
        "ms" => TimeDelta::try_milliseconds(val).unwrap_or_default(),
        "s" => TimeDelta::try_seconds(val).unwrap_or_default(),
        "m" => TimeDelta::try_minutes(val).unwrap_or_default(),
        "h" => TimeDelta::try_hours(val).unwrap_or_default(),
        "d" => TimeDelta::try_days(val).unwrap_or_default(),
        _ => TimeDelta::try_seconds(val).unwrap_or_default(),
    };

    if is_neg { -delta } else { delta }
}
