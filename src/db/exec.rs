/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{PgArgs, Tx};
use crate::store;

/// Executes an UPDATE query using the first initialized database pool.
pub async fn update<T>(table: &str, set: &str, condition: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if condition.trim().is_empty() {
        if with_deleted_at {
            format!("UPDATE {} SET {} WHERE deleted_at IS NULL", table, set)
        } else {
            format!("UPDATE {} SET {}", table, set)
        }
    } else if with_deleted_at {
        format!("UPDATE {} SET {} WHERE ({}) AND deleted_at IS NULL", table, set, condition)
    } else {
        format!("UPDATE {} SET {} WHERE {}", table, set, condition)
    };

    sqlx::query_with(&sql, args.build_inner()).execute(store::db()).await
}

/// Executes an UPDATE query on a specific database.
pub async fn update_on<T>(
    key: &str,
    table: &str,
    set: &str,
    condition: &str,
    args: PgArgs<T>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if condition.trim().is_empty() {
        if with_deleted_at {
            format!("UPDATE {} SET {} WHERE deleted_at IS NULL", table, set)
        } else {
            format!("UPDATE {} SET {}", table, set)
        }
    } else if with_deleted_at {
        format!("UPDATE {} SET {} WHERE ({}) AND deleted_at IS NULL", table, set, condition)
    } else {
        format!("UPDATE {} SET {} WHERE {}", table, set, condition)
    };

    sqlx::query_with(&sql, args.build_inner()).execute(store::db_on(key)).await
}

/// Executes an UPDATE query within a transaction.
pub async fn tx_update<T>(
    tx: &Tx,
    table: &str,
    set: &str,
    condition: &str,
    args: PgArgs<T>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if condition.trim().is_empty() {
        if with_deleted_at {
            format!("UPDATE {} SET {} WHERE deleted_at IS NULL", table, set)
        } else {
            format!("UPDATE {} SET {}", table, set)
        }
    } else if with_deleted_at {
        format!("UPDATE {} SET {} WHERE ({}) AND deleted_at IS NULL", table, set, condition)
    } else {
        format!("UPDATE {} SET {} WHERE {}", table, set, condition)
    };

    let mut lock = tx.inner.lock().await;
    let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
    sqlx::query_with(&sql, args.build_inner()).execute(&mut **inner_tx).await
}

/// Executes a query using the first initialized database pool that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute<T>(sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.build_inner()).execute(store::db()).await
}

/// Executes a query that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute_on<T>(key: &str, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.build_inner()).execute(store::db_on(key)).await
}

pub async fn tx_execute<T>(tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let mut lock = tx.inner.lock().await;
    let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
    sqlx::query_with(sql, args.build_inner()).execute(&mut **inner_tx).await
}
