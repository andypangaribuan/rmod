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

struct DbContainer {
    map: HashMap<String, &'static DbPools>,
    keys: Vec<String>,
}

static DB_STORE: OnceLock<Mutex<DbContainer>> = OnceLock::new();

fn get_db_store() -> &'static Mutex<DbContainer> {
    DB_STORE.get_or_init(|| Mutex::new(DbContainer { map: HashMap::new(), keys: Vec::new() }))
}

/// Sets the database pools for a specific key.
pub(crate) fn set_db(key: &str, write_pool: Pool<Postgres>, read_pool: Option<Pool<Postgres>>) {
    let pools = Box::leak(Box::new(DbPools { write: write_pool, read: read_pool }));

    let mut store = get_db_store().lock().unwrap();
    if store.map.contains_key(key) {
        panic!("DB Pool with key '{}' already set", key);
    }

    store.map.insert(key.to_string(), pools);
    store.keys.push(key.to_string());
}

fn get_pools(key: &str) -> &'static DbPools {
    let store = get_db_store().lock().unwrap();
    store.map.get(key).copied().unwrap_or_else(|| panic!("DB Pool with key '{}' not initialized", key))
}

fn get_first_pools() -> &'static DbPools {
    let store = get_db_store().lock().unwrap();
    let key = store.keys.first().expect("No DB Pools initialized");
    store.map.get(key).copied().unwrap()
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

/// Gets the write database pool for the first initialized key.
pub fn db1() -> &'static Pool<Postgres> {
    &get_first_pools().write
}

/// Gets the read database pool for the first initialized key.
pub fn db1_read() -> &'static Pool<Postgres> {
    let pools = get_first_pools();
    pools.read.as_ref().unwrap_or(&pools.write)
}
