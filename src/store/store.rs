/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use once_cell::sync::OnceCell;
use sqlx::{Pool, Postgres};

static DB_WRITE_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();
static DB_READ_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

/// Sets the global database pools.
pub(crate) fn set_db(write_pool: Pool<Postgres>, read_pool: Option<Pool<Postgres>>) {
    DB_WRITE_POOL
        .set(write_pool)
        .expect("DB Write Pool already set");
    if let Some(pool) = read_pool {
        DB_READ_POOL.set(pool).expect("DB Read Pool already set");
    }
}

/// Gets the write database pool.
pub fn db() -> &'static Pool<Postgres> {
    DB_WRITE_POOL.get().expect("DB Write Pool not initialized")
}

/// Gets the read database pool. Falls back to write pool if read pool is not initialized.
pub fn db_read() -> &'static Pool<Postgres> {
    DB_READ_POOL
        .get()
        .or_else(|| DB_WRITE_POOL.get())
        .expect("DB Pools not initialized")
}
