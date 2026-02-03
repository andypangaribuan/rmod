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
use chrono::{Utc, TimeZone};

#[test]
fn test_time_parse() {
    let dt = Utc.with_ymd_and_hms(2024, 2, 3, 7, 38, 42).unwrap();
    let formatted = time_parse(dt, "%Y-%m-%d %H:%M:%S");
    assert_eq!(formatted, "2024-02-03 07:38:42");
}
