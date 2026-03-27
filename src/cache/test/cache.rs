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
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_cache() {
    let group = "test_group";
    let ttl = "1s";
    let max_capacity = 10;

    // Initialize cache group natively supporting String types
    add_group::<String>(group, ttl, max_capacity);

    let key = "test_key";
    let val = "test_value".to_string();

    // Store key value pairs
    put::<String>(group, key, val.clone()).await;

    // Successfully retrieve the cached parameter
    let retrieved = get::<String>(group, key).await;
    assert_eq!(retrieved, Some(val), "The cached string match correctly");

    // Undefined key triggers None internally
    let missing_key = get::<String>(group, "not_present").await;
    assert_eq!(missing_key, None, "Should not return undefined keys");

    // Expire cache correctly
    sleep(Duration::from_millis(1200)).await;
    let expired = get::<String>(group, key).await;
    assert_eq!(expired, None, "The key should expire safely after the configured TTL terminates");
}

#[tokio::test]
async fn test_cache_unlimited_capacity() {
    let group = "test_group_unlimited";
    let ttl = "10m"; // long duration

    // Initialize group with no capacity constraints (-1) on a numeric u64 cache
    add_group::<u64>(group, ttl, -1);

    put::<u64>(group, "number", 42).await;

    let output = get::<u64>(group, "number").await;
    assert_eq!(output, Some(42), "Retrieved number from numeric cache group succeeds");
}

#[tokio::test]
async fn test_cache_ttl_reset() {
    let group = "test_group_ttl";
    let ttl = "1s";

    add_group::<String>(group, ttl, 10);

    let key = "the_key";

    // T = 0ms: Put value A
    put::<String>(group, key, "value_A".to_string()).await;

    // T = 800ms
    sleep(Duration::from_millis(800)).await;

    // Check it's still there
    let val = get::<String>(group, key).await;
    assert_eq!(val, Some("value_A".to_string()));

    // Put value B over the same key, which should reset the 1s TTL timer
    put::<String>(group, key, "value_B".to_string()).await;

    // T = 1300ms (500ms since Put B, 1300ms since Put A)
    // If the timer didn't reset, it would be dead (1300ms > 1s TTL)
    sleep(Duration::from_millis(500)).await;
    let val_b = get::<String>(group, key).await;
    assert_eq!(val_b, Some("value_B".to_string()));

    // T = 2000ms (1200ms since Put B)
    // Should now definitively be dead
    sleep(Duration::from_millis(700)).await;
    let val_c = get::<String>(group, key).await;
    assert_eq!(val_c, None);
}
