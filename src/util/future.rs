/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/future.rs"]
mod tests;

use std::future::Future;
use tokio::task::JoinSet;

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

pub async fn future_burst<T, R, F, Fut>(data: Vec<T>, max_parallel: usize, f: F) -> Vec<(usize, R)>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(usize, T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    let mut join_set = JoinSet::new();
    let mut results = Vec::with_capacity(data.len());
    let f = std::sync::Arc::new(f);
    let max_parallel = if max_parallel == 0 { 1 } else { max_parallel };

    for (idx, item) in data.into_iter().enumerate() {
        while join_set.len() >= max_parallel {
            if let Some(Ok(val)) = join_set.join_next().await {
                results.push(val);
            }
        }

        let f_clone = std::sync::Arc::clone(&f);
        join_set.spawn(async move {
            let res = f_clone(idx, item).await;
            (idx, res)
        });
    }

    while let Some(Ok(val)) = join_set.join_next().await {
        results.push(val);
    }

    results
}
