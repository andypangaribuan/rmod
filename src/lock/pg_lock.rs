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

pub(super) async fn lock(key: &str, opt_wait_ms: Option<i64>) -> Result<(sqlx::pool::PoolConnection<Postgres>, Vec<i64>), String> {
    lock_many(vec![key], opt_wait_ms).await
}

pub(super) async fn lock_many(
    keys: Vec<&str>,
    opt_wait_ms: Option<i64>,
) -> Result<(sqlx::pool::PoolConnection<Postgres>, Vec<i64>), String> {
    let pool = POOL.get().expect("Pg lock pool not initialized");
    let timeout_ms = opt_wait_ms.unwrap_or_else(|| *LOCK_TIMEOUT.get().unwrap_or(&30) as i64 * 1000) as u64;

    let mut lock_keys = Vec::new();
    for key in &keys {
        let mut hasher = SipHasher13::new_with_keys(0, 0);
        key.hash(&mut hasher);
        lock_keys.push(hasher.finish() as i64);
    }

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *conn).await.map_err(|e| e.to_string())?;

    let start = std::time::Instant::now();
    let mut acquired_keys = Vec::new();

    loop {
        let mut all_success = true;
        for &lock_key in &lock_keys {
            let result: Result<bool, sqlx::Error> =
                sqlx::query_scalar("SELECT pg_try_advisory_lock($1)").bind(lock_key).fetch_one(&mut *conn).await;

            if let Ok(true) = result {
                acquired_keys.push(lock_key);
            } else {
                all_success = false;
                break;
            }
        }

        if all_success {
            println!("Acquired pg multi-lock: {:?} (IDs: {:?}, PID: {})", keys, lock_keys, pid);
            return Ok((conn, lock_keys));
        }

        // Rolling back if not all could be acquired in this attempt
        for &lock_key in acquired_keys.iter() {
            let _: Result<bool, _> = sqlx::query_scalar("SELECT pg_advisory_unlock($1)").bind(lock_key).fetch_one(&mut *conn).await;
        }
        acquired_keys.clear();

        if start.elapsed().as_millis() as u64 >= timeout_ms {
            drop(conn);
            return Err(format!("Failed to acquire pg multi-lock for keys '{:?}' within {} ms", keys, timeout_ms));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub(super) async fn unlock(mut conn: sqlx::pool::PoolConnection<Postgres>, key: &str, lock_keys: Vec<i64>) {
    let pid_res: Result<i32, _> = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *conn).await;
    let pid = pid_res.unwrap_or(-1);

    for lock_key in lock_keys {
        let res: Result<bool, _> = sqlx::query_scalar("SELECT pg_advisory_unlock($1)").bind(lock_key).fetch_one(&mut *conn).await;
        match res {
            Ok(true) => {
                println!("Successfully unlocked pg lock item: {} (ID: {}, PID: {})", key, lock_key, pid);
            }
            Ok(false) => {
                eprintln!(
                    "Failed to release pg lock item: {} (ID: {}, PID: {}). Returned false. Detaching connection.",
                    key, lock_key, pid
                );
                let _ = conn.detach();
                return;
            }
            Err(e) => {
                eprintln!("Error executing pg unlock for item: {} (ID: {}). Error: {}. Detaching connection.", key, lock_key, e);
                let _ = conn.detach();
                return;
            }
        }
    }
}
