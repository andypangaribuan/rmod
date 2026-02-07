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
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

struct DbPools {
    write: Pool<Postgres>,
    read: Option<Pool<Postgres>>,
}

static DB_MAP: OnceLock<Mutex<HashMap<String, &'static DbPools>>> = OnceLock::new();

fn get_db_map() -> &'static Mutex<HashMap<String, &'static DbPools>> {
    DB_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sets the database pools for a specific key.
pub(crate) fn set_db(key: &str, write_pool: Pool<Postgres>, read_pool: Option<Pool<Postgres>>) {
    let pools = Box::leak(Box::new(DbPools { write: write_pool, read: read_pool }));

    let mut map = get_db_map().lock().unwrap();
    if map.contains_key(key) {
        panic!("DB Pool with key '{}' already set", key);
    }
    map.insert(key.to_string(), pools);
}

fn get_pools(key: &str) -> &'static DbPools {
    let map = get_db_map().lock().unwrap();
    map.get(key).copied().unwrap_or_else(|| panic!("DB Pool with key '{}' not initialized", key))
}

/// Gets the write database pool for a specific key.
pub fn db(key: &str) -> &'static Pool<Postgres> {
    &get_pools(key).write
}

/// Gets the read database pool for a specific key. Falls back to write pool if read pool is not initialized.
pub fn db_read(key: &str) -> &'static Pool<Postgres> {
    let pools = get_pools(key);
    pools.read.as_ref().unwrap_or(&pools.write)
}
