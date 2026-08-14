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
        if (self.pg_conn.is_some() || self.redis_val.is_some())
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            let conn = self.pg_conn.take();
            let redis_val = self.redis_val.take();
            let pg_lock_keys = std::mem::take(&mut self.pg_lock_keys);
            let key = self.key.clone();

            let log_ctx = crate::clog::get_current_ctx();

            handle.spawn(async move {
                let fut = async move {
                    if let Some(c) = conn {
                        super::pg_lock::dist_unlock(c, &key, pg_lock_keys).await;
                    }

                    if let Some(v) = redis_val {
                        super::redis_lock::dist_unlock(&key, &v).await;
                    }
                };

                if let Some(ctx) = log_ctx {
                    crate::clog::LOG_CTX.scope(std::cell::RefCell::new(ctx), fut).await;
                } else {
                    fut.await;
                }
            });
        }
    }
}

impl DistLock {
    pub async fn unlock(mut self) {
        self.perform_unlock().await;
    }

    pub(super) async fn perform_unlock(&mut self) {
        if let Some(conn) = self.pg_conn.take() {
            let key = self.key.clone();
            let pg_lock_keys = std::mem::take(&mut self.pg_lock_keys);
            super::pg_lock::dist_unlock(conn, &key, pg_lock_keys).await;
        }

        if let Some(val) = self.redis_val.take() {
            let key = self.key.clone();
            super::redis_lock::dist_unlock(&key, &val).await;
        }
    }
}
