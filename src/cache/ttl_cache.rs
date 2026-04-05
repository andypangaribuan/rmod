/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::time;
use moka::future::Cache;
use std::{
    any::Any,
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

static GROUPS: OnceLock<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>> = OnceLock::new();

pub(super) fn get_groups() -> &'static RwLock<HashMap<String, Box<dyn Any + Send + Sync>>> {
    GROUPS.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn add_group_ttl<T: Clone + Send + Sync + 'static>(group_name: &str, ttl: &str, maximum_capacity: i64) {
    let mut builder = Cache::builder().time_to_live(time::to_duration(ttl));

    if maximum_capacity > 0 {
        builder = builder.max_capacity(maximum_capacity as u64);
    }

    let cache: Cache<String, T> = builder.build();
    let mut groups = get_groups().write().unwrap_or_else(|poisoned| poisoned.into_inner());
    groups.insert(group_name.to_string(), Box::new(cache));
}

pub async fn put_ttl<T: Clone + Send + Sync + 'static>(group_name: &str, key: &str, value: T) {
    let cache = {
        let groups = get_groups().read().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(c) = groups.get(group_name) { c.downcast_ref::<Cache<String, T>>().cloned() } else { None }
    };

    if let Some(c) = cache {
        c.insert(key.to_string(), value).await;
    }
}

pub async fn get_ttl<T: Clone + Send + Sync + 'static>(group_name: &str, key: &str) -> Option<T> {
    let cache = {
        let groups = get_groups().read().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(c) = groups.get(group_name) { c.downcast_ref::<Cache<String, T>>().cloned() } else { None }
    };

    if let Some(c) = cache { c.get(key).await } else { None }
}
