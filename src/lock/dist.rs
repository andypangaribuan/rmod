/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::OnceLock;

pub enum DistLockType {
    Pg,
    Redis,
}

pub static LOCK_TYPE: OnceLock<DistLockType> = OnceLock::new();

pub struct DistLock {
    key: String,
    pg_conn: Option<sqlx::pool::PoolConnection<sqlx::Postgres>>,
    redis_val: Option<String>,
}

impl DistLock {
    pub fn unlock(mut self) {
        self.perform_unlock();
    }

    fn perform_unlock(&mut self) {
        if let Some(conn) = self.pg_conn.take() {
            let key = self.key.clone();
            tokio::spawn(async move {
                crate::lock::pg_lock::unlock(conn, &key).await;
            });
        }
        if let Some(val) = self.redis_val.take() {
            let key = self.key.clone();
            tokio::spawn(async move {
                crate::lock::redis_lock::unlock(&key, &val).await;
            });
        }
    }
}

impl Drop for DistLock {
    fn drop(&mut self) {
        self.perform_unlock();
    }
}

pub struct LockOptions {
    ttl_ms: Option<i64>,
    wait_ms: Option<i64>,
}

pub fn opt() -> LockOptions {
    LockOptions { ttl_ms: None, wait_ms: None }
}

impl LockOptions {
    pub fn ttl<T: crate::time::ToDuration>(mut self, duration: T) -> Self {
        self.ttl_ms = Some(duration.to_duration().as_millis() as i64);
        self
    }

    pub fn wait<T: crate::time::ToDuration>(mut self, duration: T) -> Self {
        self.wait_ms = Some(duration.to_duration().as_millis() as i64);
        self
    }
}

pub async fn lock(key: &str, opt: Option<LockOptions>) -> Result<DistLock, String> {
    let t = LOCK_TYPE.get().ok_or("Distribution lock not initialized")?;
    let (ttl_ms, wait_ms) = match opt {
        Some(o) => (o.ttl_ms, o.wait_ms),
        None => (None, None),
    };

    match t {
        DistLockType::Pg => {
            let conn = crate::lock::pg_lock::lock(key, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: Some(conn), redis_val: None })
        }
        DistLockType::Redis => {
            let val = crate::lock::redis_lock::lock(key, ttl_ms, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: None, redis_val: Some(val) })
        }
    }
}
