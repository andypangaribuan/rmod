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
use crate::arcx;

#[test]
fn test_arcx_new_get() {
    let a = ArcX::new(10);
    assert_eq!(a.get(), 10);
}

#[test]
fn test_arcx_set() {
    let a = ArcX::new(10);
    a.set(20);
    assert_eq!(a.get(), 20);
}

#[test]
fn test_arcx_lock() {
    let a = ArcX::new(10);
    {
        let mut lock = a.lock();
        *lock += 5;
    }
    assert_eq!(a.get(), 15);
}

#[test]
fn test_arcx_clone() {
    let a = ArcX::new(10);
    let b = a.clone();
    a.set(20);
    assert_eq!(b.get(), 20);
}

#[test]
fn test_arcx_debug() {
    let a = ArcX::new(10);
    assert_eq!(format!("{:?}", a), "ArcX(10)");
}

#[test]
fn test_arcx_macro() {
    let a = arcx!(10);
    assert_eq!(a.get(), 10);
}

#[tokio::test]
async fn test_arcx_with_future_pool() {
    let a = ArcX::new(0);
    let mut pool = crate::util::FuturePool::new();

    for i in 0..10 {
        let b = a.clone();
        pool.add(i, async move {
            let mut lock = b.lock();
            *lock += 1;
        });
    }

    pool.join_all().await;
    assert_eq!(a.get(), 10);
}

#[tokio::test]
async fn test_arcx_with_future_burst() {
    let a = ArcX::new(0);
    let data = vec![1; 10]; // 10 items

    let a_clone = a.clone();
    crate::util::future_burst(data, 2, move |_, _| {
        let b = a_clone.clone();
        async move {
            let mut lock = b.lock();
            *lock += 1;
        }
    })
    .await;

    assert_eq!(a.get(), 10);
}
