/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::model::RedisLockConfig;
use crate::config::DbConfig;

pub async fn pg_lock(config: &DbConfig) -> Result<(), String> {
    pg_lock_initialize(config).await
}

pub async fn redis_lock(config: &RedisLockConfig) -> Result<(), String> {
    redis_lock_initialize(config).await
}

pub async fn pg_lock_initialize(config: &DbConfig) -> Result<(), String> {
    if config.host.is_empty() {
        return Err("host cannot be empty".to_string());
    }
    if config.port == 0 {
        return Err("port must be greater than 0".to_string());
    }
    if config.database.is_empty() {
        return Err("database cannot be empty".to_string());
    }
    if config.username.is_empty() {
        return Err("username cannot be empty".to_string());
    }

    crate::lock::pg_lock_initialize(config).await
}

pub async fn redis_lock_initialize(config: &RedisLockConfig) -> Result<(), String> {
    if config.host.is_empty() {
        return Err("host cannot be empty".to_string());
    }
    if config.port == 0 {
        return Err("port must be greater than 0".to_string());
    }

    crate::lock::redis_lock_initialize(config).await
}
