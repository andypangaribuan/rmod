/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::get_groups;
use crate::time;
use moka::{Expiry, future::Cache};
use std::time::{Duration, Instant};

pub struct CustomExpiry;

impl<K, T> Expiry<K, (T, Duration)> for CustomExpiry {
    fn expire_after_create(&self, _key: &K, value: &(T, Duration), _current_time: Instant) -> Option<Duration> {
        Some(value.1)
    }

    fn expire_after_read(
        &self,
        _key: &K,
        _value: &(T, Duration),
        _current_time: Instant,
        duration_until_expiry: Option<Duration>,
        _last_modified_at: Instant,
    ) -> Option<Duration> {
        duration_until_expiry
    }

    fn expire_after_update(
        &self,
        _key: &K,
        value: &(T, Duration),
        _current_time: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        Some(value.1)
    }
}

pub fn add_group_exp<T: Clone + Send + Sync + 'static>(group_name: &str, maximum_capacity: i64) {
    let mut builder = Cache::builder().expire_after(CustomExpiry);

    if maximum_capacity > 0 {
        builder = builder.max_capacity(maximum_capacity as u64);
    }

    let cache: Cache<String, (T, Duration)> = builder.build();
    let mut groups = get_groups().write().unwrap_or_else(|poisoned| poisoned.into_inner());
    groups.insert(group_name.to_string(), Box::new(cache));
}

pub async fn put_exp<T: Clone + Send + Sync + 'static>(group_name: &str, key: &str, value: T, ttl: &str) {
    let duration = time::to_duration(ttl);
    let cache = {
        let groups = get_groups().read().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(c) = groups.get(group_name) { c.downcast_ref::<Cache<String, (T, Duration)>>().cloned() } else { None }
    };

    if let Some(c) = cache {
        c.insert(key.to_string(), (value, duration)).await;
    }
}

pub async fn get_exp<T: Clone + Send + Sync + 'static>(group_name: &str, key: &str) -> Option<T> {
    let cache = {
        let groups = get_groups().read().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(c) = groups.get(group_name) { c.downcast_ref::<Cache<String, (T, Duration)>>().cloned() } else { None }
    };

    if let Some(c) = cache { c.get(key).await.map(|v| v.0) } else { None }
}
