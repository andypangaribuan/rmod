/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::model::DbConfig;
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::time::Duration;

pub fn timezone(val: &str) {
    crate::store::update_timezone(val.to_string());
}

pub async fn db_setup(
    key: &str,
    write: DbConfig,
    read: Option<DbConfig>,
    updated_at: i64,
    state: &str,
    conn_str: &str,
) -> Result<(), sqlx::Error> {
    let write_pool = create_db_pool(&write).await?;
    let read_pool = if let Some(config) = read { Some(create_db_pool(&config).await?) } else { None };
    crate::store::set_db(key, write_pool, read_pool, updated_at, state, conn_str);
    Ok(())
}

async fn create_db_pool(config: &DbConfig) -> Result<Pool<Postgres>, sqlx::Error> {
    let connect_options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database)
        .options([("search_path", config.schema.as_deref().unwrap_or("public"))]);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout.unwrap_or(30) as u64))
        .idle_timeout(Duration::from_secs(config.idle_timeout.unwrap_or(10) as u64))
        .connect_with(connect_options)
        .await?;

    Ok(pool)
}
