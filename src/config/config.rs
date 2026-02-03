/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Sets up a PostgreSQL connection pool and saves it to the global store.
pub async fn db_setup(url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?;

    crate::store::set_db(pool.clone());
    Ok(pool)
}
