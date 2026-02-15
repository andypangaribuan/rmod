/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/conv.rs"]
mod tests;

use chrono::{DateTime, Utc};
use std::time::Duration;

/// Formats a DateTime<Utc> into a string with the given format.
pub fn time_parse(dt: DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

pub fn to_duration(duration: &str) -> Duration {
    let mut val = duration.to_string();
    let mut unit = "s";

    if val.ends_with("ms") {
        unit = "ms";
        val = val[..val.len() - 2].to_string();
    } else if val.ends_with('s') {
        unit = "s";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('m') {
        unit = "m";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('h') {
        unit = "h";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('d') {
        unit = "d";
        val = val[..val.len() - 1].to_string();
    }

    let val = val.parse::<u64>().unwrap_or(0);

    match unit {
        "ms" => Duration::from_millis(val),
        "s" => Duration::from_secs(val),
        "m" => Duration::from_secs(val * 60),
        "h" => Duration::from_secs(val * 3600),
        "d" => Duration::from_secs(val * 86400),
        _ => Duration::from_secs(val),
    }
}
