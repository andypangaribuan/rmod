/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::config::RedisLockConfig;
use std::{sync::OnceLock, time::Duration};

static REDIS_CLIENT: OnceLock<redis::Client> = OnceLock::new();
static LOCK_TTL: OnceLock<i64> = OnceLock::new();

pub(crate) async fn initialize_dist_lock(config: &RedisLockConfig) -> Result<(), String> {
    let auth = if let Some(pass) = &config.password {
        if let Some(user) = &config.username { format!("{}:{}@", user, pass) } else { format!(":{}@", pass) }
    } else {
        "".to_string()
    };

    let url = format!("redis://{}{}:{}/{}", auth, config.host, config.port, config.database);
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    REDIS_CLIENT.set(client).map_err(|_| "Redis Client already initialized".to_string())?;
    LOCK_TTL.set(config.ttl.unwrap_or(30000)).ok();
    super::LOCK_TYPE.set(super::DistLockType::Redis).ok();
    Ok(())
}

pub(super) async fn dist_lock(key: &str, opt_ttl: Option<i64>, opt_wait_ms: Option<i64>) -> Result<String, String> {
    let client = REDIS_CLIENT.get().expect("Redis lock client not initialized");
    let ttl = opt_ttl.unwrap_or_else(|| *LOCK_TTL.get().unwrap_or(&30000));
    let wait_ms = opt_wait_ms.unwrap_or(30000) as u64;

    let start_lock_time = std::time::Instant::now();
    let mut conn: redis::aio::MultiplexedConnection = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let err_msg = e.to_string();
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let mut payload_map = serde_json::json!({
                "key": key,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "error": err_msg,
            });
            let clean_st = crate::clog::clean_stacktrace(&bt_str);
            if !clean_st.trim().is_empty() {
                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
            }
            crate::clog::log_dist_lock_redis_lock(key, duration_ms, 500, payload_map.to_string());
            return Err(err_msg);
        }
    };

    let val = crate::uid::new();

    let start = std::time::Instant::now();
    let mut current_backoff_ms = 10;
    let max_backoff_ms = 500;

    loop {
        let result: redis::RedisResult<bool> =
            redis::cmd("SET").arg(key).arg(&val).arg("NX").arg("PX").arg(ttl).query_async(&mut conn).await;

        if let Ok(true) = result {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let payload_json = serde_json::json!({
                "key": key,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "val": val,
            })
            .to_string();
            crate::clog::log_dist_lock_redis_lock(key, duration_ms, 200, payload_json);
            return Ok(val);
        }

        if start.elapsed().as_millis() as u64 >= wait_ms {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let err_msg = format!("Failed to acquire redis lock for key '{}' within {} ms", key, wait_ms);
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let mut payload_map = serde_json::json!({
                "key": key,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "error": err_msg,
            });
            let clean_st = crate::clog::clean_stacktrace(&bt_str);
            if !clean_st.trim().is_empty() {
                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
            }
            crate::clog::log_dist_lock_redis_lock(key, duration_ms, 500, payload_map.to_string());
            return Err(err_msg);
        }

        let sleep_ms = rand::random_range((current_backoff_ms / 2)..=current_backoff_ms);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;

        current_backoff_ms = (current_backoff_ms * 2).min(max_backoff_ms);
    }
}

pub(super) async fn dist_lock_many(keys: Vec<&str>, opt_ttl: Option<i64>, opt_wait_ms: Option<i64>) -> Result<String, String> {
    let client = REDIS_CLIENT.get().expect("Redis lock client not initialized");
    let ttl = opt_ttl.unwrap_or_else(|| *LOCK_TTL.get().unwrap_or(&30000));
    let wait_ms = opt_wait_ms.unwrap_or(30000) as u64;

    let mut sorted_keys = keys.clone();
    sorted_keys.sort_unstable();
    sorted_keys.dedup();

    let action_name = keys.join(",");
    let start_lock_time = std::time::Instant::now();

    let mut conn: redis::aio::MultiplexedConnection = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let err_msg = e.to_string();
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let mut payload_map = serde_json::json!({
                "keys": keys,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "error": err_msg,
            });
            let clean_st = crate::clog::clean_stacktrace(&bt_str);
            if !clean_st.trim().is_empty() {
                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
            }
            crate::clog::log_dist_lock_redis_lock(&action_name, duration_ms, 500, payload_map.to_string());
            return Err(err_msg);
        }
    };

    let val = crate::uid::new();
    let start = std::time::Instant::now();
    let mut current_backoff_ms = 10;
    let max_backoff_ms = 500;

    loop {
        let mut acquired_keys = Vec::new();
        let mut all_success = true;

        for k in &sorted_keys {
            let result: redis::RedisResult<bool> =
                redis::cmd("SET").arg(*k).arg(&val).arg("NX").arg("PX").arg(ttl).query_async(&mut conn).await;

            if let Ok(true) = result {
                acquired_keys.push(*k);
            } else {
                all_success = false;
                break;
            }
        }

        if all_success {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let payload_json = serde_json::json!({
                "keys": keys,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "val": val,
            })
            .to_string();
            crate::clog::log_dist_lock_redis_lock(&action_name, duration_ms, 200, payload_json);
            return Ok(val);
        }

        for k in acquired_keys {
            let script = redis::Script::new(
                r#"
                    if redis.call("get",KEYS[1]) == ARGV[1] then
                        return redis.call("del",KEYS[1])
                    else
                        return 0
                    end
                    "#,
            );
            let _: redis::RedisResult<i32> = script.key(k).arg(&val).invoke_async(&mut conn).await;
        }

        if start.elapsed().as_millis() as u64 >= wait_ms {
            let duration_ms = start_lock_time.elapsed().as_millis() as i32;
            let err_msg = format!("Failed to acquire redis multi-lock for keys '{:?}' within {} ms", keys, wait_ms);
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let mut payload_map = serde_json::json!({
                "keys": keys,
                "action": "LOCK",
                "ttl": ttl,
                "wait_ms": opt_wait_ms,
                "error": err_msg,
            });
            let clean_st = crate::clog::clean_stacktrace(&bt_str);
            if !clean_st.trim().is_empty() {
                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
            }
            crate::clog::log_dist_lock_redis_lock(&action_name, duration_ms, 500, payload_map.to_string());
            return Err(err_msg);
        }

        let sleep_ms = rand::random_range((current_backoff_ms / 2)..=current_backoff_ms);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;

        current_backoff_ms = (current_backoff_ms * 2).min(max_backoff_ms);
    }
}

pub(super) async fn dist_unlock(key: &str, val: &str) {
    let start_time = std::time::Instant::now();
    let Some(client) = REDIS_CLIENT.get() else {
        return;
    };
    let keys: Vec<&str> = key.split(',').collect();

    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            let mut all_ok = true;
            for k in &keys {
                let script = redis::Script::new(
                    r#"
                        if redis.call("get",KEYS[1]) == ARGV[1] then
                            return redis.call("del",KEYS[1])
                        else
                            return 0
                        end
                        "#,
                );
                let res: redis::RedisResult<i32> = script.key(*k).arg(val).invoke_async(&mut conn).await;
                if res.is_err() {
                    all_ok = false;
                }
            }
            let duration_ms = start_time.elapsed().as_millis() as i32;
            if all_ok {
                let payload_json = serde_json::json!({
                    "keys": keys,
                    "action": "UNLOCK",
                    "val": val,
                })
                .to_string();
                crate::clog::log_dist_lock_redis_unlock(key, duration_ms, 200, payload_json);
            } else {
                tracing::error!("Failed to unlock redis lock for key '{}'", key);
                let bt = std::backtrace::Backtrace::force_capture();
                let bt_str = format!("{}", bt);
                let mut payload_map = serde_json::json!({
                    "keys": keys,
                    "action": "UNLOCK",
                    "val": val,
                    "error": "Failed to unlock one or more redis keys",
                });
                let clean_st = crate::clog::clean_stacktrace(&bt_str);
                if !clean_st.trim().is_empty() {
                    payload_map["stacktrace"] = serde_json::Value::String(clean_st);
                }
                crate::clog::log_dist_lock_redis_unlock(key, duration_ms, 500, payload_map.to_string());
            }
        }
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as i32;
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let mut payload_map = serde_json::json!({
                "keys": keys,
                "action": "UNLOCK",
                "val": val,
                "error": e.to_string(),
            });
            let clean_st = crate::clog::clean_stacktrace(&bt_str);
            if !clean_st.trim().is_empty() {
                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
            }
            crate::clog::log_dist_lock_redis_unlock(key, duration_ms, 500, payload_map.to_string());
        }
    }
}
