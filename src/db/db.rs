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

pub use sqlx::Arguments;
pub use sqlx::FromRow;
pub use sqlx::postgres::PgArguments;

#[macro_export]
macro_rules! db_args {
    ($($x:expr),*) => {
        {
            use $crate::db::Arguments;
            let mut args = $crate::db::PgArguments::default();
            $(
                args.add($x);
            )*
            args
        }
    };
}

pub use db_args as args;

/// Executes a query and returns a single row.
pub async fn fetch_one<T>(key: &str, sql: &str, args: PgArguments) -> Result<T, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_one(store::db_read(key)).await
}

/// Executes a query and returns an optional row.
pub async fn fetch_optional<T>(key: &str, sql: &str, args: PgArguments) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_optional(store::db_read(key)).await
}

/// Executes a query and returns all rows.
pub async fn fetch_all<T>(key: &str, sql: &str, args: PgArguments) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_all(store::db_read(key)).await
}

/// Executes a query using the first initialized database pool and returns a single row.
pub async fn fetch_one_first<T>(sql: &str, args: PgArguments) -> Result<T, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_one(store::db_read_first()).await
}

/// Executes a query using the first initialized database pool and returns an optional row.
pub async fn fetch_optional_first<T>(sql: &str, args: PgArguments) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_optional(store::db_read_first()).await
}

/// Executes a query using the first initialized database pool and returns all rows.
pub async fn fetch_all_first<T>(sql: &str, args: PgArguments) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as_with(sql, args).fetch_all(store::db_read_first()).await
}

/// Executes a query that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute(key: &str, sql: &str, args: PgArguments) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args).execute(store::db(key)).await
}

/// Executes a query using the first initialized database pool that does not return rows (e.g., INSERT, UPDATE, DELETE).
pub async fn execute_first(sql: &str, args: PgArguments) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_with(sql, args).execute(store::db_first()).await
}
