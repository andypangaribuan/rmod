/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{
    config::{self, DbConfig},
    lock::{lock, lock_many, opt},
};
use std::{net::TcpStream, time::Instant};

async fn initialize_pg() -> Result<(), String> {
    let config = DbConfig {
        host: "127.0.0.1".to_string(),
        port: 15432,
        database: "dist-lock-db".to_string(), // from res/compose/.env
        schema: Some("public".to_string()),
        username: "rmod".to_string(),
        password: "E5BEWREN1N7w12G9U73JKPf8rQst4WQPMHKLqdNdG1gGabPQi9".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: Some(5),
        idle_timeout: Some(5),
        lock_timeout: Some(30),
    };

    config::pg_lock(&config).await
}

#[tokio::test]
async fn test_dist_lock_pg() {
    if TcpStream::connect("127.0.0.1:15432").is_err() {
        println!("Postgres container is not running on port 15432. Skipping test_dist_lock_pg.");
        return;
    }

    if let Err(e) = initialize_pg().await {
        println!("Failed to initialize pg lock, it might be already initialized by another test. Error: {}", e);
    }

    let key = "test_pg_dist_lock_key";

    // 1. Test lock acquisition
    let lock1 = lock(key, None).await.expect("Failed to acquire first lock");

    // 2. Test lock collision (should fail or block and timeout)
    let start = Instant::now();
    let lock2_result = lock(key, Some(opt().wait(std::time::Duration::from_millis(500)))).await;
    assert!(lock2_result.is_err(), "Second lock should fail to acquire since it's held by lock1");
    assert!(start.elapsed().as_millis() >= 400, "Should have waited for timeout");

    // 3. Test unlocking
    lock1.unlock().await;

    // 4. Test re-acquisition after unlock
    let lock3 = lock(key, Some(opt().wait(std::time::Duration::from_millis(100)))).await.expect("Failed to acquire lock after unlock");
    lock3.unlock().await;
}

#[tokio::test]
async fn test_dist_lock_many_pg() {
    if TcpStream::connect("127.0.0.1:15432").is_err() {
        println!("Postgres container is not running on port 15432. Skipping test_dist_lock_many_pg.");
        return;
    }

    if let Err(e) = initialize_pg().await {
        println!("Failed to initialize pg lock, it might be already initialized by another test. Error: {}", e);
    }

    let keys = vec!["test_pg_dist_lock_multi_key_1", "test_pg_dist_lock_multi_key_2"];

    // 1. Test lock many acquisition
    let lock1 = lock_many(keys.clone(), None).await.expect("Failed to acquire first multi-lock");

    // 2. Test lock collision on one of the keys
    let start = Instant::now();
    let lock2_result = lock(keys[0], Some(opt().wait(std::time::Duration::from_millis(500)))).await;
    assert!(lock2_result.is_err(), "Second lock should fail to acquire since it's held by lock1");
    assert!(start.elapsed().as_millis() >= 400, "Should have waited for timeout");

    // 3. Test unlocking
    lock1.unlock().await;

    // 4. Test re-acquisition after unlock
    let lock3 =
        lock_many(keys, Some(opt().wait(std::time::Duration::from_millis(100)))).await.expect("Failed to acquire multi-lock after unlock");
    lock3.unlock().await;
}
