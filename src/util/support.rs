/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use chrono;
use std::collections::HashSet;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        println!("{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z"), format!($($arg)*));
    };
}

pub fn collect_unique<T, K, F>(items: Vec<T>, f: F) -> Vec<K>
where
    K: std::hash::Hash + Eq + Clone,
    F: Fn(T) -> K,
{
    let mut seen = HashSet::new();
    items.into_iter().map(f).filter(|k| seen.insert(k.clone())).collect()
}

pub fn have_in<T: PartialEq>(value: T, list: Vec<T>) -> bool {
    list.contains(&value)
}
