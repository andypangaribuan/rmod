/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use siphasher::sip::SipHasher13;
use sqlx::Postgres;
use std::{
    hash::{Hash, Hasher},
    sync::OnceLock,
};

static POOL: OnceLock<sqlx::PgPool> = OnceLock::new();
static LOCK_TIMEOUT: OnceLock<i16> = OnceLock::new();

pub(crate) async fn initialize(config: &crate::config::DbConfig) -> Result<(), String> {
    let connect_options = sqlx::postgres::PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database)
        .options([("search_path", config.schema.as_deref().unwrap_or("public"))]);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout.unwrap_or(30) as u64))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout.unwrap_or(10) as u64))
        .connect_with(connect_options)
        .await
        .map_err(|e| e.to_string())?;

    POOL.set(pool).map_err(|_| "Pg Lock Pool already initialized".to_string())?;
    LOCK_TIMEOUT.set(config.lock_timeout.unwrap_or(30)).ok();
    super::LOCK_TYPE.set(super::DistLockType::Pg).ok();
    Ok(())
}

pub(super) async fn lock(key: &str, opt_wait_ms: Option<i64>) -> Result<(sqlx::pool::PoolConnection<Postgres>, Vec<(i32, i32)>), String> {
    lock_many(vec![key], opt_wait_ms).await
}

pub(super) async fn lock_many(
    keys: Vec<&str>,
    opt_wait_ms: Option<i64>,
) -> Result<(sqlx::pool::PoolConnection<Postgres>, Vec<(i32, i32)>), String> {
    let pool = POOL.get().expect("Pg lock pool not initialized");
    let timeout_ms = opt_wait_ms.unwrap_or_else(|| *LOCK_TIMEOUT.get().unwrap_or(&30) as i64 * 1000) as u64;

    let mut lock_keys = Vec::new();
    for key in &keys {
        let mut hasher = SipHasher13::new_with_keys(0, 0);
        key.hash(&mut hasher);
        let hash = hasher.finish();
        // Split 64-bit hash into two 32-bit integers for Postgres compat
        let k1 = (hash >> 32) as i32;
        let k2 = (hash & 0xFFFFFFFF) as i32;
        lock_keys.push((k1, k2));
    }

    lock_keys.sort_unstable();
    lock_keys.dedup();

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let start = std::time::Instant::now();

    loop {
        // Begin a transaction. This pins the connection in PgBouncer (Transaction mode).
        let _ = sqlx::query("BEGIN").execute(&mut *conn).await.ok();

        let mut all_success = true;
        for &(k1, k2) in &lock_keys {
            // Using xact_lock guarantees automatic release on COMMIT/ROLLBACK
            let result: Result<bool, sqlx::Error> =
                sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1, $2)").bind(k1).bind(k2).fetch_one(&mut *conn).await;

            if let Ok(true) = result {
                // Lock acquired for this key.
            } else {
                all_success = false;
                break;
            }
        }

        if all_success {
            // Success! The active transaction holds all requested locks safely.
            return Ok((conn, lock_keys));
        }

        // If we fail to acquire all, rollback the transaction to release partial locks instantly.
        let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();

        if start.elapsed().as_millis() as u64 >= timeout_ms {
            drop(conn);
            return Err(format!("Failed to acquire pg multi-lock for keys '{:?}' within {} ms", keys, timeout_ms));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

pub(super) async fn unlock(mut conn: sqlx::pool::PoolConnection<Postgres>, _key: &str, _lock_keys: Vec<(i32, i32)>) {
    // Simply rolling back the transaction will release all xact_locks and unpin PgBouncer.
    let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
}
