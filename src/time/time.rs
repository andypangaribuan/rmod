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
    if duration.is_empty() {
        return Duration::from_secs(0);
    }

    let (val_str, unit) = if let Some(stripped) = duration.strip_suffix("ms") {
        (stripped, "ms")
    } else if let Some(stripped) = duration.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = duration.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = duration.strip_suffix('h') {
        (stripped, "h")
    } else if let Some(stripped) = duration.strip_suffix('d') {
        (stripped, "d")
    } else {
        (duration, "s")
    };

    let val = val_str.parse::<u64>().unwrap_or(0);

    match unit {
        "ms" => Duration::from_millis(val),
        "s" => Duration::from_secs(val),
        "m" => Duration::from_secs(val * 60),
        "h" => Duration::from_secs(val * 3600),
        "d" => Duration::from_secs(val * 86400),
        _ => Duration::from_secs(val),
    }
}
