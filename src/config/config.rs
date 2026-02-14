/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub schema: Option<String>,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Option<i16>,
    pub idle_timeout: Option<i16>,
}

pub async fn db_setup(key: &str, updated_at: i64, write: DbConfig, read: Option<DbConfig>) -> Result<(), sqlx::Error> {
    let write_pool = create_db_pool(&write).await?;
    let read_pool = if let Some(config) = read { Some(create_db_pool(&config).await?) } else { None };
    crate::store::set_db(key, updated_at, write_pool, read_pool);
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
