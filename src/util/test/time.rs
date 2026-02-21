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

#[tokio::test]
async fn test_sleep() {
    let start = std::time::Instant::now();
    let duration = Duration::from_millis(100);
    sleep(duration).await;
    let elapsed = start.elapsed();
    assert!(elapsed >= duration);
}

#[tokio::test]
async fn test_sleep_str() {
    let start = std::time::Instant::now();
    sleep("100ms").await;
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(100));
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
