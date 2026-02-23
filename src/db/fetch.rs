/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::store;

use super::{PgArgs, Tx};
pub use sqlx::FromRow;

/// Executes a query using the first initialized database pool and returns an optional row.
pub async fn fetch<T>(sql: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let force_rw = args.is_force_rw();
    let use_read = !force_rw && store::db_is_read_real();
    let pool = if use_read { store::db_read() } else { store::db() };

    let mut res = sqlx::query_as_with(sql, args.build_inner()).fetch_optional(pool).await?;

    if use_read
        && let Some(validate) = args.opt.as_ref().and_then(|o| o.validate.as_ref())
        && !validate(&res)
    {
        res = sqlx::query_as_with(sql, args.build_inner()).fetch_optional(store::db()).await?;
    }

    Ok(res)
}

/// Executes a query using the first initialized database pool and returns all rows.
pub async fn fetch_all<T>(sql: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let force_rw = args.is_force_rw();
    let use_read = !force_rw && store::db_is_read_real();
    let pool = if use_read { store::db_read() } else { store::db() };

    let mut res = sqlx::query_as_with(sql, args.build_inner()).fetch_all(pool).await?;

    if use_read
        && let Some(validate_all) = args.opt.as_ref().and_then(|o| o.validate_all.as_ref())
        && !validate_all(&res)
    {
        res = sqlx::query_as_with(sql, args.build_inner()).fetch_all(store::db()).await?;
    }

    Ok(res)
}

/// Executes a query and returns an optional row.
pub async fn fetch_on<T>(key: &str, sql: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let force_rw = args.is_force_rw();
    let use_read = !force_rw && store::db_is_read_real_on(key);
    let pool = if use_read { store::db_read_on(key) } else { store::db_on(key) };

    let mut res = sqlx::query_as_with(sql, args.build_inner()).fetch_optional(pool).await?;

    if use_read
        && let Some(validate) = args.opt.as_ref().and_then(|o| o.validate.as_ref())
        && !validate(&res)
    {
        res = sqlx::query_as_with(sql, args.build_inner()).fetch_optional(store::db_on(key)).await?;
    }

    Ok(res)
}

/// Executes a query and returns all rows.
pub async fn fetch_all_on<T>(key: &str, sql: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let force_rw = args.is_force_rw();
    let use_read = !force_rw && store::db_is_read_real_on(key);
    let pool = if use_read { store::db_read_on(key) } else { store::db_on(key) };

    let mut res = sqlx::query_as_with(sql, args.build_inner()).fetch_all(pool).await?;

    if use_read
        && let Some(validate_all) = args.opt.as_ref().and_then(|o| o.validate_all.as_ref())
        && !validate_all(&res)
    {
        res = sqlx::query_as_with(sql, args.build_inner()).fetch_all(store::db_on(key)).await?;
    }

    Ok(res)
}

/// Executes a query using the first initialized database pool that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute<T>(sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.build_inner()).execute(store::db()).await
}

/// Executes a query that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute_on<T>(key: &str, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.build_inner()).execute(store::db_on(key)).await
}

pub async fn tx_fetch<T>(tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let mut lock = tx.inner.lock().await;
    let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
    sqlx::query_as_with(sql, args.build_inner()).fetch_optional(&mut **inner_tx).await
}

pub async fn tx_fetch_all<T>(tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    let mut lock = tx.inner.lock().await;
    let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
    sqlx::query_as_with(sql, args.build_inner()).fetch_all(&mut **inner_tx).await
}

pub async fn tx_execute<T>(tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let mut lock = tx.inner.lock().await;
    let inner_tx = lock.as_mut().expect("Transaction already committed or rolled back");
    sqlx::query_with(sql, args.build_inner()).execute(&mut **inner_tx).await
}

// Executes a query and returns a single row.
// pub async fn fetch_one<T>(key: &str, sql: &str, args: PgArguments) -> Result<T, sqlx::Error>
// where
//     T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
// {
//     sqlx::query_as_with(sql, args).fetch_one(store::db_read(key)).await
// }

// Executes a query using the first initialized database pool and returns a single row.
// pub async fn fetch1_one<T>(sql: &str, args: PgArguments) -> Result<T, sqlx::Error>
// where
//     T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
// {
//     sqlx::query_as_with(sql, args).fetch_one(store::db1_read()).await
// }
