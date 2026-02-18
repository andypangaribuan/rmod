/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::future::Future;
use tokio::task::JoinSet;

#[cfg(test)]
#[path = "test/future.rs"]
mod tests;

pub struct FuturePool<K, V> {
    join_set: JoinSet<(K, V)>,
}

impl<K, V> FuturePool<K, V>
where
    K: Send + 'static,
    V: Send + 'static,
{
    pub fn new() -> Self {
        Self { join_set: JoinSet::new() }
    }

    pub fn add<F>(&mut self, key: K, fut: F)
    where
        F: Future<Output = V> + Send + 'static,
    {
        self.join_set.spawn(async move { (key, fut.await) });
    }

    pub async fn join_all(&mut self) -> Vec<(K, V)> {
        let mut results = Vec::new();
        while let Some(res) = self.join_set.join_next().await {
            if let Ok(val) = res {
                results.push(val);
            }
        }
        results
    }

    pub async fn join_next(&mut self) -> Option<(K, V)> {
        while let Some(res) = self.join_set.join_next().await {
            if let Ok(val) = res {
                return Some(val);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.join_set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.join_set.is_empty()
    }
}

impl<K, V> Default for FuturePool<K, V>
where
    K: Send + 'static,
    V: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
