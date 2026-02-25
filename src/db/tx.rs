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
#[path = "test/tx.rs"]
mod tests;

use crate::store;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

pub struct Tx {
    pub(crate) inner: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
    pub(crate) committed: Arc<AtomicBool>,
    pub(crate) rolled_back: Arc<AtomicBool>,
}

impl Tx {
    pub(crate) fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(tx))),
            committed: Arc::new(AtomicBool::new(false)),
            rolled_back: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn commit(&self) -> Result<(), sqlx::Error> {
        if self.committed.load(Ordering::SeqCst) || self.rolled_back.load(Ordering::SeqCst) {
            return Ok(());
        }

        let mut lock = self.inner.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
            self.committed.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn rollback(&self) {
        if self.committed.load(Ordering::SeqCst) || self.rolled_back.load(Ordering::SeqCst) {
            return;
        }

        self.rolled_back.store(true, Ordering::SeqCst);
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut lock = inner.lock().await;
            if let Some(tx) = lock.take() {
                let _ = tx.rollback().await;
            }
        });
    }
}

pub async fn tx() -> Result<Tx, sqlx::Error> {
    let pool = store::db();
    let tx = pool.begin().await?;
    Ok(Tx::new(tx))
}

pub async fn tx_on(key: &str) -> Result<Tx, sqlx::Error> {
    let pool = store::db_on(key);
    let tx = pool.begin().await?;
    Ok(Tx::new(tx))
}
