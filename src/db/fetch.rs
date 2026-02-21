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

use super::PgArgs;
pub use sqlx::FromRow;

/// Executes a query using the first initialized database pool and returns an optional row.
pub async fn fetch<T>(sql: &str, args: PgArgs) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let pool = if args.is_force_rw() { store::db() } else { store::db_read() };
    sqlx::query_as_with(sql, args.inner).fetch_optional(pool).await
}

/// Executes a query using the first initialized database pool and returns all rows.
pub async fn fetch_all<T>(sql: &str, args: PgArgs) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let pool = if args.is_force_rw() { store::db() } else { store::db_read() };
    sqlx::query_as_with(sql, args.inner).fetch_all(pool).await
}

/// Executes a query and returns an optional row.
pub async fn fetch_on<T>(key: &str, sql: &str, args: PgArgs) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let pool = if args.is_force_rw() { store::db_on(key) } else { store::db_read_on(key) };
    sqlx::query_as_with(sql, args.inner).fetch_optional(pool).await
}

/// Executes a query and returns all rows.
pub async fn fetch_all_on<T>(key: &str, sql: &str, args: PgArgs) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let pool = if args.is_force_rw() { store::db_on(key) } else { store::db_read_on(key) };
    sqlx::query_as_with(sql, args.inner).fetch_all(pool).await
}

/// Executes a query using the first initialized database pool that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute(sql: &str, args: PgArgs) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.inner).execute(store::db()).await
}

/// Executes a query that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute_on(key: &str, sql: &str, args: PgArgs) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args.inner).execute(store::db_on(key)).await
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
