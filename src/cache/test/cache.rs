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
