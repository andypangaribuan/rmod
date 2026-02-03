/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use sqlx::{Pool, Postgres};
use once_cell::sync::OnceCell;

static DB_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

/// Sets the global database pool.
/// Panics if the pool is already set.
pub(crate) fn set_db(pool: Pool<Postgres>) {
    DB_POOL.set(pool).expect("DB Pool already set");
}

/// Gets the global database pool.
/// Panics if the pool has not been initialized.
pub fn db() -> &'static Pool<Postgres> {
    DB_POOL.get().expect("DB Pool not initialized")
}
