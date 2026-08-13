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
    pub(crate) id: String,
    pub(crate) key: Option<String>,
    pub(crate) inner: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
    pub(crate) committed: Arc<AtomicBool>,
    pub(crate) rolled_back: Arc<AtomicBool>,
}

impl Tx {
    pub(crate) fn new(tx: Transaction<'static, Postgres>, key: Option<String>) -> Self {
        Self {
            id: crate::uid::new(),
            key,
            inner: Arc::new(Mutex::new(Some(tx))),
            committed: Arc::new(AtomicBool::new(false)),
            rolled_back: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn commit(&self) -> Result<(), sqlx::Error> {
        if self.committed.load(Ordering::SeqCst) || self.rolled_back.load(Ordering::SeqCst) {
            return Ok(());
        }

        let mut lock = self.inner.lock().await;
        if let Some(tx) = lock.take() {
            let start = std::time::Instant::now();
            let res = tx.commit().await;
            let duration_ms = start.elapsed().as_millis() as i32;

            match res {
                Ok(_) => {
                    self.committed.store(true, Ordering::SeqCst);
                    crate::clog::log_tx_commit(&self.id, self.key.as_deref(), duration_ms, 200, None, None);
                    Ok(())
                }
                Err(e) => {
                    let bt = std::backtrace::Backtrace::force_capture();
                    let bt_str = format!("{}", bt);
                    let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
                    crate::clog::log_tx_commit(&self.id, self.key.as_deref(), duration_ms, 500, Some(&e.to_string()), stacktrace);
                    Err(e)
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn rollback(&self) {
        if self.committed.load(Ordering::SeqCst) || self.rolled_back.load(Ordering::SeqCst) {
            return;
        }

        self.rolled_back.store(true, Ordering::SeqCst);
        let inner = Arc::clone(&self.inner);
        let id = self.id.clone();
        let key = self.key.clone();

        tokio::spawn(async move {
            let mut lock = inner.lock().await;
            if let Some(tx) = lock.take() {
                let start = std::time::Instant::now();
                let res = tx.rollback().await;
                let duration_ms = start.elapsed().as_millis() as i32;

                match res {
                    Ok(_) => {
                        crate::clog::log_tx_rollback(&id, key.as_deref(), duration_ms, 200, None, None);
                    }
                    Err(e) => {
                        let bt = std::backtrace::Backtrace::force_capture();
                        let bt_str = format!("{}", bt);
                        let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
                        crate::clog::log_tx_rollback(&id, key.as_deref(), duration_ms, 500, Some(&e.to_string()), stacktrace);
                    }
                }
            }
        });
    }
}

pub async fn tx() -> Result<Tx, sqlx::Error> {
    let start = std::time::Instant::now();
    let pool = store::db();
    let res = pool.begin().await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match res {
        Ok(tx) => {
            let tx_obj = Tx::new(tx, None);
            crate::clog::log_tx_begin(&tx_obj.id, None, duration_ms, 200, None, None);
            Ok(tx_obj)
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_tx_begin("", None, duration_ms, 500, Some(&e.to_string()), stacktrace);
            Err(e)
        }
    }
}

pub async fn tx_on(key: &str) -> Result<Tx, sqlx::Error> {
    let start = std::time::Instant::now();
    let pool = store::db_on(key);
    let res = pool.begin().await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match res {
        Ok(tx) => {
            let tx_obj = Tx::new(tx, Some(key.to_string()));
            crate::clog::log_tx_begin(&tx_obj.id, Some(key), duration_ms, 200, None, None);
            Ok(tx_obj)
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_tx_begin("", Some(key), duration_ms, 500, Some(&e.to_string()), stacktrace);
            Err(e)
        }
    }
}
