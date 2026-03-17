/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub struct DistLock {
    pub(super) key: String,
    pub(super) pg_conn: Option<sqlx::pool::PoolConnection<sqlx::Postgres>>,
    pub(super) pg_lock_keys: Vec<(i32, i32)>,
    pub(super) redis_val: Option<String>,
}

pub struct LockOptions {
    pub(super) ttl_ms: Option<i64>,
    pub(super) wait_ms: Option<i64>,
}
