/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

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
    pub lock_timeout: Option<i16>,
}

pub struct RedisLockConfig {
    pub host: String,
    pub port: u16,
    pub database: i64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ttl: Option<i64>,
}
