/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::db::{FromRow, PgArgs, Tx};
use std::marker::PhantomData;

use super::build_select_sql;

pub struct Repo<T> {
    pub table_name: &'static str,
    pub columns: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> Repo<T>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
{
    pub const fn new(table_name: &'static str, columns: &'static str) -> Self {
        Self { table_name, columns, _phantom: PhantomData }
    }

    /// Fetches an optional row using the first initialized database pool.
    pub async fn fetch(&self, where_clause: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::fetch::<T>(&sql, args).await
    }

    /// Fetches all rows using the first initialized database pool.
    pub async fn fetch_all(&self, where_clause: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::fetch_all::<T>(&sql, args).await
    }

    /// Fetches exactly one row using the first initialized database pool.
    pub async fn query(&self, where_clause: &str, args: PgArgs<T>) -> Result<T, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::query::<T>(&sql, args).await
    }

    /// Fetches an optional row from a specific database.
    pub async fn fetch_on(&self, key: &str, where_clause: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::fetch_on::<T>(key, &sql, args).await
    }

    /// Fetches all rows from a specific database.
    pub async fn fetch_all_on(&self, key: &str, where_clause: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::fetch_all_on::<T>(key, &sql, args).await
    }

    /// Fetches exactly one row from a specific database.
    pub async fn query_on(&self, key: &str, where_clause: &str, args: PgArgs<T>) -> Result<T, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::query_on::<T>(key, &sql, args).await
    }

    /// Executes a query using the first initialized database pool (e.g., INSERT, UPDATE, DELETE).
    pub async fn execute(&self, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::execute(sql, args).await
    }

    /// Executes a query on a specific database.
    pub async fn execute_on(&self, key: &str, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::execute_on(key, sql, args).await
    }

    /// Executes an UPDATE query using the first initialized database pool.
    pub async fn update(&self, set: &str, condition: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::update(self.table_name, set, condition, args).await
    }

    /// Executes an UPDATE query on a specific database.
    pub async fn update_on(
        &self,
        key: &str,
        set: &str,
        condition: &str,
        args: PgArgs<T>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::update_on(key, self.table_name, set, condition, args).await
    }

    /// Automatically generates and executes an INSERT statement for this table.
    pub async fn insert(&self, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let table_name = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(self.table_name);
        let count = self.columns.split(',').count();
        let placeholders = (1..=count).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, self.columns, placeholders);
        crate::db::execute(&sql, args).await
    }

    pub async fn insert_on(&self, key: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let table_name = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(self.table_name);
        let count = self.columns.split(',').count();
        let placeholders = (1..=count).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, self.columns, placeholders);
        crate::db::execute_on(key, &sql, args).await
    }

    pub async fn tx_fetch(&self, tx: &Tx, where_clause: &str, args: PgArgs<T>) -> Result<Option<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::tx_fetch::<T>(tx, &sql, args).await
    }

    pub async fn tx_fetch_all(&self, tx: &Tx, where_clause: &str, args: PgArgs<T>) -> Result<Vec<T>, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::tx_fetch_all::<T>(tx, &sql, args).await
    }

    pub async fn tx_query(&self, tx: &Tx, where_clause: &str, args: PgArgs<T>) -> Result<T, sqlx::Error> {
        let sql = build_select_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::tx_query::<T>(tx, &sql, args).await
    }

    pub async fn tx_execute(&self, tx: &Tx, sql: &str, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::tx_execute(tx, sql, args).await
    }

    pub async fn tx_insert(&self, tx: &Tx, args: PgArgs<T>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let table_name = args.opt.as_ref().and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(self.table_name);
        let count = self.columns.split(',').count();
        let placeholders = (1..=count).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, self.columns, placeholders);
        crate::db::tx_execute(tx, &sql, args).await
    }

    pub async fn tx_update(
        &self,
        tx: &Tx,
        set: &str,
        condition: &str,
        args: PgArgs<T>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        crate::db::tx_update(tx, self.table_name, set, condition, args).await
    }

    /// Fetches the count of rows using the first initialized database pool.
    pub async fn count(&self, where_clause: &str, args: PgArgs<T>) -> Result<i64, sqlx::Error> {
        let sql = super::build_count_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::count::<T>(&sql, args).await
    }

    /// Fetches the count of rows from a specific database.
    pub async fn count_on(&self, key: &str, where_clause: &str, args: PgArgs<T>) -> Result<i64, sqlx::Error> {
        let sql = super::build_count_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::count_on::<T>(key, &sql, args).await
    }

    pub async fn tx_count(&self, tx: &Tx, where_clause: &str, args: PgArgs<T>) -> Result<i64, sqlx::Error> {
        let sql = super::build_count_sql(self.table_name, where_clause, args.opt.as_ref());
        crate::db::tx_count::<T>(tx, &sql, args).await
    }
}
