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
    let decoded = decode_timestamp_base62(&encoded);

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
