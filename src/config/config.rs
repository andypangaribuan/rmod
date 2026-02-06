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

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub schema: Option<String>,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Sets up the PostgreSQL connection pools and saves them to the global store.
pub async fn db_setup(write: DbConfig, read: Option<DbConfig>) -> Result<(), sqlx::Error> {
    let write_pool = create_db_pool(&write).await?;
    let mut read_pool = None;

    if let Some(config) = read {
        read_pool = Some(create_db_pool(&config).await?);
    }

    crate::store::set_db(write_pool, read_pool);
    Ok(())
}

async fn create_db_pool(config: &DbConfig) -> Result<Pool<Postgres>, sqlx::Error> {
    let mut connect_options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database);

    if let Some(schema) = &config.schema {
        connect_options = connect_options.options([("search_path", schema)]);
    }

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_with(connect_options)
        .await?;

    Ok(pool)
}
