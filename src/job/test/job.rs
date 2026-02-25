/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::job;
use futures_util::future::BoxFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn test_handler() -> BoxFuture<'static, ()> {
    Box::pin(async {
        COUNTER.fetch_add(1, Ordering::SeqCst);
    })
}

#[tokio::test]
async fn test_job_execution() {
    // Reset counter just in case
    COUNTER.store(0, Ordering::SeqCst);

    // Add a job that runs every 100ms
    job::add("100ms", test_handler, true, false);

    // Start jobs
    job::start();

    // Wait for a bit (e.g., 350ms to get ~3-4 ticks)
    // interval(100ms) ticks immediately on start, then at 100, 200, 300.
    tokio::time::sleep(Duration::from_millis(350)).await;

    let count = COUNTER.load(Ordering::SeqCst);
    println!("Job ran {} times", count);

    // Should have run at least 3 times
    assert!(count >= 3);
}

static COUNTER_NON_EVERY: AtomicUsize = AtomicUsize::new(0);

fn test_handler_non_every() -> BoxFuture<'static, ()> {
    Box::pin(async {
        COUNTER_NON_EVERY.fetch_add(1, Ordering::SeqCst);
    })
}

#[tokio::test]
async fn test_job_non_every_execution() {
    COUNTER_NON_EVERY.store(0, Ordering::SeqCst);

    // Add a job that runs after 100ms (is_every = false)
    job::add("100ms", test_handler_non_every, false, false);

    job::start();

    // Wait 350ms
    // 0ms: run (1), sleep 100ms
    // 100ms: run (2), sleep 100ms
    // 200ms: run (3), sleep 100ms
    // 300ms: run (4), sleep 100ms
    tokio::time::sleep(Duration::from_millis(350)).await;

    let count = COUNTER_NON_EVERY.load(Ordering::SeqCst);
    println!("Non-every Job ran {} times", count);

    assert!(count >= 3);
}
