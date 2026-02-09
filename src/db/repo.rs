/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::db::{FromRow, PgArgs};
use std::marker::PhantomData;

use super::build_select_sql;

pub struct Repo<T> {
    pub table_name: &'static str,
    pub columns: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> Repo<T>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    pub const fn new(table_name: &'static str, columns: &'static str) -> Self {
        Self { table_name, columns, _phantom: PhantomData }
    }

    /// Fetches an optional row using the first initialized database pool.
    pub async fn fetch(&self, where_clause: &str, args: PgArgs) -> Result<Option<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause);
        crate::db::fetch::<T>(&sql, args).await
    }

    /// Fetches all rows using the first initialized database pool.
    pub async fn fetch_all(&self, where_clause: &str, args: PgArgs) -> Result<Vec<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause);
        crate::db::fetch_all::<T>(&sql, args).await
    }

    /// Fetches an optional row from a specific database.
    pub async fn get(&self, key: &str, where_clause: &str, args: PgArgs) -> Result<Option<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause);
        crate::db::get::<T>(key, &sql, args).await
    }

    /// Fetches all rows from a specific database.
    pub async fn get_all(&self, key: &str, where_clause: &str, args: PgArgs) -> Result<Vec<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause);
        crate::db::get_all::<T>(key, &sql, args).await
    }

    /// Executes a query using the first initialized database pool (e.g., INSERT, UPDATE, DELETE).
    pub async fn execute(&self, sql: &str, args: PgArgs) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::execute(sql, args).await
    }

    /// Executes a query on a specific database.
    pub async fn perform(&self, key: &str, sql: &str, args: PgArgs) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::perform(key, sql, args).await
    }

    /// Automatically generates and executes an INSERT statement for this table.
    pub async fn insert(&self, args: PgArgs) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let count = self.columns.split(',').count();
        let placeholders = (1..=count).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", self.table_name, self.columns, placeholders);
        crate::db::execute(&sql, args).await
    }
}
