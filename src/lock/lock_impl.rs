/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::DistLock;

impl Drop for DistLock {
    fn drop(&mut self) {
        self.perform_unlock();
    }
}

impl DistLock {
    pub fn unlock(mut self) {
        self.perform_unlock();
    }

    pub(super) fn perform_unlock(&mut self) {
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
