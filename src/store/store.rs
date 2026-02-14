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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static DB_WITH_DELETED_AT: AtomicBool = AtomicBool::new(false);

pub fn set_db_with_deleted_at(val: bool) {
    DB_WITH_DELETED_AT.store(val, Ordering::Relaxed);
}

pub(crate) fn get_db_with_deleted_at() -> bool {
    DB_WITH_DELETED_AT.load(Ordering::Relaxed)
}

struct DbPools {
    write: Pool<Postgres>,
    read: Option<Pool<Postgres>>,
}

struct DbContainer {
    keys: Vec<String>,
    map: HashMap<String, &'static DbPools>,
    updated_at: HashMap<String, i64>,
}

static DB_STORE: OnceLock<Mutex<DbContainer>> = OnceLock::new();

fn get_db_store() -> &'static Mutex<DbContainer> {
    DB_STORE.get_or_init(|| Mutex::new(DbContainer { keys: Vec::new(), map: HashMap::new(), updated_at: HashMap::new() }))
}

/// Sets the database pools for a specific key.
pub(crate) fn set_db(key: &str, updated_at: i64, write_pool: Pool<Postgres>, read_pool: Option<Pool<Postgres>>) {
    let pools = Box::leak(Box::new(DbPools { write: write_pool, read: read_pool }));

    let mut store = get_db_store().lock().unwrap();
    if store.map.contains_key(key) {
        panic!("DB Pool with key '{}' already set", key);
    }

    store.map.insert(key.to_string(), pools);
    store.updated_at.insert(key.to_string(), updated_at);
    store.keys.push(key.to_string());
}

fn get_pools(key: &str) -> &'static DbPools {
    let store = get_db_store().lock().unwrap();
    store.map.get(key).copied().unwrap_or_else(|| panic!("DB Pool with key '{}' not initialized", key))
}

pub(crate) fn db_exists(key: &str) -> bool {
    let store = get_db_store().lock().unwrap();
    store.map.contains_key(key)
}

fn get_first_pools() -> &'static DbPools {
    let store = get_db_store().lock().unwrap();
    let key = store.keys.first().expect("No DB Pools initialized");
    store.map.get(key).copied().unwrap()
}

/// Gets the write database pool for the first initialized key.
pub(crate) fn db() -> &'static Pool<Postgres> {
    &get_first_pools().write
}

/// Gets the read database pool for the first initialized key.
pub(crate) fn db_read() -> &'static Pool<Postgres> {
    let pools = get_first_pools();
    pools.read.as_ref().unwrap_or(&pools.write)
}

/// Gets the write database pool for a specific key.
pub(crate) fn db_on(key: &str) -> &'static Pool<Postgres> {
    &get_pools(key).write
}

/// Gets the read database pool for a specific key. Falls back to write pool if read pool is not initialized.
pub(crate) fn db_read_on(key: &str) -> &'static Pool<Postgres> {
    let pools = get_pools(key);
    pools.read.as_ref().unwrap_or(&pools.write)
}
