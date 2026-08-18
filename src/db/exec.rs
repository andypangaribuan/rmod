/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{PgArgs, Tx, function::replace_table_name};
use crate::store;

/// Executes an UPDATE query using the first initialized database pool.
pub async fn update<T>(table: &str, set: &str, condition: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let original_table = table;
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if let Some(full_query) = args.opt.as_ref().and_then(|o| o.full_query.as_ref()) {
        if original_table != table { replace_table_name(full_query, original_table, table) } else { full_query.to_string() }
    } else if condition.trim().is_empty() {
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

    let sql_log = sql.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_db_update(None, &sql_log, args_opt, async move {
        sqlx::query_with(&sql, args.build_inner()).execute(store::db()).await
    })
    .await
}

/// Executes an UPDATE query on a specific database.
pub async fn update_on<T>(
    key: &str,
    table: &str,
    set: &str,
    condition: &str,
    args: PgArgs<T>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let original_table = table;
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if let Some(full_query) = args.opt.as_ref().and_then(|o| o.full_query.as_ref()) {
        if original_table != table { replace_table_name(full_query, original_table, table) } else { full_query.to_string() }
    } else if condition.trim().is_empty() {
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

    let sql_log = sql.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_db_update(Some(key), &sql_log, args_opt, async move {
        sqlx::query_with(&sql, args.build_inner()).execute(store::db_on(key)).await
    })
    .await
}

/// Executes an UPDATE query within a transaction.
pub async fn tx_update<T>(
    tx: &Tx,
    table: &str,
    set: &str,
    condition: &str,
    args: PgArgs<T>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let original_table = table;
    let table = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table);
    let with_deleted_at = args.opt.as_ref().and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);

    let sql = if let Some(full_query) = args.opt.as_ref().and_then(|o| o.full_query.as_ref()) {
        if original_table != table { replace_table_name(full_query, original_table, table) } else { full_query.to_string() }
    } else if condition.trim().is_empty() {
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

    let sql_log = sql.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_tx_db_update(tx, &sql_log, args_opt, async move {
        let mut lock = tx.inner.lock().await;
        let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
        sqlx::query_with(&sql, args.build_inner()).execute(&mut **inner_tx).await
    })
    .await
}

/// Executes a query using the first initialized database pool that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute<T>(sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let sql_query = args.opt.as_ref().and_then(|o| o.full_query.as_ref()).map(|s| s.to_string()).unwrap_or_else(|| sql.to_string());
    let sql_log = sql_query.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_db_execute(None, &sql_log, args_opt, async move {
        sqlx::query_with(&sql_query, args.build_inner()).execute(store::db()).await
    })
    .await
}

/// Executes a query that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute_on<T>(key: &str, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let sql_query = args.opt.as_ref().and_then(|o| o.full_query.as_ref()).map(|s| s.to_string()).unwrap_or_else(|| sql.to_string());
    let sql_log = sql_query.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_db_execute(Some(key), &sql_log, args_opt, async move {
        sqlx::query_with(&sql_query, args.build_inner()).execute(store::db_on(key)).await
    })
    .await
}

pub async fn tx_execute<T>(tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let sql_query = args.opt.as_ref().and_then(|o| o.full_query.as_ref()).map(|s| s.to_string()).unwrap_or_else(|| sql.to_string());
    let sql_log = sql_query.clone();
    let debug_args = if args.is_empty() { None } else { Some(args.values().to_vec()) };
    let args_opt = debug_args.as_deref();
    super::function::log_tx_db_execute(tx, &sql_log, args_opt, async move {
        let mut lock = tx.inner.lock().await;
        let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
        sqlx::query_with(&sql_query, args.build_inner()).execute(&mut **inner_tx).await
    })
    .await
}
