/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::*;
use chrono::{TimeZone, Utc};

#[test]
fn test_time_parse() {
    let dt = Utc.with_ymd_and_hms(2024, 2, 3, 7, 38, 42).unwrap();
    let formatted = time_parse(dt, "%Y-%m-%d %H:%M:%S");
    assert_eq!(formatted, "2024-02-03 07:38:42");
}

#[test]
fn test_to_duration() {
    assert_eq!(to_duration("10s"), Duration::from_secs(10));
    assert_eq!(to_duration("100ms"), Duration::from_millis(100));
    assert_eq!(to_duration("5m"), Duration::from_secs(5 * 60));
    assert_eq!(to_duration("2h"), Duration::from_secs(2 * 3600));
    assert_eq!(to_duration("1d"), Duration::from_secs(86400));
    assert_eq!(to_duration("123"), Duration::from_secs(123)); // default to seconds
    assert_eq!(to_duration(""), Duration::from_secs(0));
    assert_eq!(to_duration("invalid"), Duration::from_secs(0));
    assert_eq!(to_duration("10x"), Duration::from_secs(0)); // suffix 'x' not handled -> "10x" parse u64 fails -> 0
}
