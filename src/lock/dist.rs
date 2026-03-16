/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{DistLock, LockOptions};
use std::sync::OnceLock;

pub(super) enum DistLockType {
    Pg,
    Redis,
}

pub(super) static LOCK_TYPE: OnceLock<DistLockType> = OnceLock::new();

impl DistLock {
    pub fn unlock(mut self) {
        self.perform_unlock();
    }

    fn perform_unlock(&mut self) {
        if let Some(conn) = self.pg_conn.take() {
            let key = self.key.clone();
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    super::pg_lock::unlock(conn, &key).await;
                });
            } else {
                let _ = conn.detach();
            }
        }
        if let Some(val) = self.redis_val.take() {
            let key = self.key.clone();
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    super::redis_lock::unlock(&key, &val).await;
                });
            }
        }
    }
}

impl Drop for DistLock {
    fn drop(&mut self) {
        self.perform_unlock();
    }
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
            let conn = super::pg_lock::lock(key, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: Some(conn), redis_val: None })
        }
        DistLockType::Redis => {
            let val = super::redis_lock::lock(key, ttl_ms, wait_ms).await?;
            Ok(DistLock { key: key.to_string(), pg_conn: None, redis_val: Some(val) })
        }
    }
}
