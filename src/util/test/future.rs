/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::util::future::FuturePool;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_future_pool() {
    let mut pool = FuturePool::new();

    pool.add("task1", async {
        sleep(Duration::from_millis(100)).await;
        "result1"
    });

    pool.add("task2", async {
        sleep(Duration::from_millis(50)).await;
        "result2"
    });

    let results = pool.join_all().await;
    assert_eq!(results.len(), 2);

    // Check if both results are present (order might vary due to concurrency)
    let mut keys: Vec<&str> = results.iter().map(|(k, _)| *k).collect();
    keys.sort();
    assert_eq!(keys, vec!["task1", "task2"]);
}

#[tokio::test]
async fn test_future_pool_incremental() {
    let mut pool = FuturePool::new();

    pool.add(1, async {
        sleep(Duration::from_millis(30)).await;
        "a"
    });

    pool.add(2, async {
        sleep(Duration::from_millis(10)).await;
        "b"
    });

    // Task 2 should finish first
    let res1 = pool.join_next().await.unwrap();
    assert_eq!(res1, (2, "b"));

    let res2 = pool.join_next().await.unwrap();
    assert_eq!(res2, (1, "a"));

    assert!(pool.join_next().await.is_none());
}
