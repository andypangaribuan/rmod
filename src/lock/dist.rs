/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/dist.rs"]
mod tests;

use super::{DistLock, LockOptions};
use std::sync::OnceLock;

pub(super) enum DistLockType {
    Pg,
    Redis,
}

pub(super) static LOCK_TYPE: OnceLock<DistLockType> = OnceLock::new();

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

pub async fn dist(key: &str, opt: Option<LockOptions>) -> Result<DistLock, String> {
    let t = LOCK_TYPE.get().ok_or("Distribution lock not initialized")?;
    let (ttl_ms, wait_ms) = match opt {
        Some(o) => (o.ttl_ms, o.wait_ms),
        None => (None, None),
    };

    match t {
        DistLockType::Pg => {
            let (conn, lock_keys) = super::pg_lock::dist_lock(key, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: Some(conn), pg_lock_keys: lock_keys, redis_val: None })
        }
        DistLockType::Redis => {
            let val = super::redis_lock::dist_lock(key, ttl_ms, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: None, pg_lock_keys: vec![], redis_val: Some(val) })
        }
    }
}

pub async fn dist_many(keys: Vec<&str>, opt: Option<LockOptions>) -> Result<DistLock, String> {
    let t = LOCK_TYPE.get().ok_or("Distribution lock not initialized")?;
    let wait_ms = match opt {
        Some(o) => o.wait_ms,
        None => None,
    };

    match t {
        DistLockType::Pg => {
            let (conn, lock_keys) = super::pg_lock::dist_lock_many(keys.clone(), wait_ms).await?;
            Ok(DistLock { key: keys.join(","), pg_conn: Some(conn), pg_lock_keys: lock_keys, redis_val: None })
        }
        DistLockType::Redis => Err("Redis dist multi-lock not implemented".to_string()),
    }
}
