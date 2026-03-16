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
    crate::lock::LOCK_TYPE.set(crate::lock::DistLockType::Pg).ok();
    Ok(())
}

pub(super) async fn lock(key: &str, opt_wait_ms: Option<i64>) -> Result<sqlx::pool::PoolConnection<Postgres>, String> {
    let pool = POOL.get().expect("Pg lock pool not initialized");
    let timeout_ms = opt_wait_ms.unwrap_or_else(|| *LOCK_TIMEOUT.get().unwrap_or(&30) as i64 * 1000) as u64;

    let mut hasher = SipHasher13::new();
    key.hash(&mut hasher);
    let lock_key = hasher.finish() as i64;

    let start = std::time::Instant::now();
    loop {
        let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

        let result: Result<(bool,), sqlx::Error> =
            sqlx::query_as("SELECT pg_try_advisory_lock($1)").bind(lock_key).fetch_one(&mut *conn).await;

        if let Ok((true,)) = result {
            return Ok(conn);
        }

        if start.elapsed().as_millis() as u64 >= timeout_ms {
            return Err(format!("Failed to acquire pg advisory lock for key '{}' within {} ms", key, timeout_ms));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub(super) async fn unlock(mut conn: sqlx::pool::PoolConnection<Postgres>, key: &str) {
    let mut hasher = SipHasher13::new();
    key.hash(&mut hasher);
    let lock_key = hasher.finish() as i64;

    let res: Result<(bool,), _> = sqlx::query_as("SELECT pg_advisory_unlock($1)").bind(lock_key).fetch_one(&mut *conn).await;

    // If the unlock query fails, detach the connection to ensure it is closed and the session lock is released,
    // rather than returning a locked connection to the pool.
    if res.is_err() {
        let _ = conn.detach();
    }
}
