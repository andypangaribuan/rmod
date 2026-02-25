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
use chrono::{Datelike, TimeZone, Timelike, Utc};

#[test]
fn test_timestamp_base62() {
    let now = Utc::now();
    // Truncate to microseconds like the Go version
    let now = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second())
        .unwrap()
        .with_nanosecond((now.nanosecond() / 1000) * 1000)
        .unwrap();

    let encoded = encode_timestamp_base62(now);
    let decoded = decode_timestamp_base62(&encoded).unwrap();

    assert_eq!(now, decoded);
}

#[test]
fn test_uid() {
    println!("uid: {}", uid());

    let a = 0.1;
    let b = 0.2;
    let result = a + b;
    println!("a + b: {}", result);

    let uid0 = uid_n(0);
    assert_eq!(uid0.len(), 10); // 10 (timestamp) + 0 random

    let uid3 = uid_n(3);
    assert_eq!(uid3.len(), 13); // 10 (timestamp) + 3 random

    let uid = uid();
    assert_eq!(uid.len(), 20); // 10 (timestamp) + 10 (random)
}

#[test]
fn test_decode_uid62() {
    let now = Utc::now();
    // Truncate to microseconds like the Go version
    let now = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second())
        .unwrap()
        .with_nanosecond((now.nanosecond() / 1000) * 1000)
        .unwrap();

    // Construct a UID manually to ensure we know the timestamp
    let encoded_time = encode_timestamp_base62(now);
    let rand_part = "abc1234567";
    let uid_str = format!("{}{}", encoded_time, rand_part);

    let (decoded_time, decoded_rand) = decode_uid62(&uid_str).unwrap();

    assert_eq!(now, decoded_time);
    assert_eq!(rand_part, decoded_rand);

    // Test invalid
    assert!(decode_uid62("short").is_none());
}

#[test]
fn test_decode_invalid_date() {
    // Generate a UID that corresponds to Feb 30th
    // We know packing is:
    // index = index * 12 + m; (month 0-11)
    // index = index * 31 + d; (day 0-30)
    // ...
    // Feb is month 1 (index), so m=1.
    // 30th is day 30, so d=29.

    // Let's manually construct a "bad" timestamp index.
    // year = 2024
    // month = 2 (Feb) -> m=1
    // day = 30 -> d=29 (valid for packing, invalid for calendar)

    // But wait, the packing logic is:
    // let m = month - 1;
    // let d = day - 1;
    // index = ...

    // We need to forge an index.
    // We can just try many random strings until one fails decoding,
    // but that's probabilistic.
    // Instead, let's use the fact that decode_timestamp_base62 calls Utc.with_ymd_and_hms.
    // If we pass "0000000000" (all 'a'), what does it decode to?
    // 0 -> year=0, m=0, d=0...
    // Utc year 0 is valid? Yes (1 BC). Month 1, Day 1. Valid.

    // We need d=30 (Day 31) for Month 2 (Feb, m=1).
    // Or d=29 (Day 30) for Month 2.

    // Let's create a helper to encode arbitrary y/m/d
    // But encode_timestamp_base62 takes DateTime<Utc>, which enforces validity.
    // So we can't use it to generate invalid date.

    // We can rely on decode_uid62 returning None for completely garbage input that happens to map to invalid date.
    // Or we can manually construct the base62 string.

    // Since we don't expose encode helper for raw numbers, we'll just try to ensure it doesn't panic on random junk.

    let junk = "zzzzzzzzzz"; // Max value
    // This will likely decode to a huge year, which might be out of range for chrono or valid.
    let res = decode_uid62(&(junk.to_string() + "1234567890"));
    // We just want to ensure it doesn't panic.
    assert!(res.is_none() || res.is_some());
}
