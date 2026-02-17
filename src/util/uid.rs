/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use rand::RngExt;

const BASE62_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const BASE62: u64 = 62;

pub fn uid() -> String {
    uid_n(10)
}

pub fn uid_n(len: usize) -> String {
    let rand_id = get_random(len, std::str::from_utf8(BASE62_ALPHABET).unwrap());
    encode_timestamp_base62(Utc::now()) + &rand_id
}

pub fn decode_uid62(uid: &str) -> Option<(DateTime<Utc>, String)> {
    if uid.len() < 10 {
        return None;
    }

    let time_part = &uid[..10];
    let rand_part = &uid[10..];

    let time_id = decode_timestamp_base62(time_part);
    Some((time_id, rand_part.to_string()))
}

fn encode_to_base62(mut n: u64, length: usize) -> String {
    let mut out = vec![0u8; length];
    for i in (0..length).rev() {
        out[i] = BASE62_ALPHABET[(n % BASE62) as usize];
        n /= BASE62;
    }
    String::from_utf8(out).unwrap_or_default()
}

fn encode_timestamp_base62(t: DateTime<Utc>) -> String {
    let year = t.year();
    let month = t.month();
    let day = t.day();
    let hour = t.hour();
    let minute = t.minute();
    let second = t.second();
    let micro = t.nanosecond() / 1000;

    let m = month - 1;
    let d = day - 1;

    let mut index: u64 = year as u64;
    index = index * 12 + m as u64;
    index = index * 31 + d as u64;
    index = index * 24 + hour as u64;
    index = index * 60 + minute as u64;
    index = index * 60 + second as u64;
    index = index * 1_000_000 + micro as u64;

    encode_to_base62(index, 10)
}

fn decode_from_base62(s: &str) -> u64 {
    let mut n: u64 = 0;
    let bytes = s.as_bytes();
    for &ch in bytes {
        let mut pos: u64 = 0;
        for j in 0..BASE62 {
            if BASE62_ALPHABET[j as usize] == ch {
                pos = j;
                break;
            }
        }
        n = n * BASE62 + pos;
    }
    n
}

fn decode_timestamp_base62(code: &str) -> DateTime<Utc> {
    let mut n = decode_from_base62(code);

    let micro = (n % 1_000_000) as u32;
    n /= 1_000_000;

    let second = (n % 60) as u32;
    n /= 60;

    let minute = (n % 60) as u32;
    n /= 60;

    let hour = (n % 24) as u32;
    n /= 24;

    let d = (n % 31) as u32;
    n /= 31;

    let m = (n % 12) as u32;
    n /= 12;

    let year = n as i32;

    let month = m + 1;
    let day = d + 1;

    Utc.with_ymd_and_hms(year, month, day, hour, minute, second).unwrap().with_nanosecond(micro * 1000).unwrap()
}

fn get_random(length: usize, value: &str) -> String {
    if length == 0 || value.is_empty() {
        return "".to_string();
    }

    let mut rng = rand::rng();
    let bytes = value.as_bytes();
    let max = bytes.len();
    let mut res = String::with_capacity(length);

    for _ in 0..length {
        let idx = rng.random_range(0..max);
        res.push(bytes[idx] as char);
    }

    res
}

#[cfg(test)]
mod tests {
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
}
