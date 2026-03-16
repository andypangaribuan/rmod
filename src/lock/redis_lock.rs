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

pub(crate) async fn initialize(config: &RedisLockConfig) -> Result<(), String> {
    let auth = if let Some(pass) = &config.password {
        if let Some(user) = &config.username { format!("{}:{}@", user, pass) } else { format!(":{}@", pass) }
    } else {
        "".to_string()
    };

    let url = format!("redis://{}{}:{}/{}", auth, config.host, config.port, config.database);
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    REDIS_CLIENT.set(client).map_err(|_| "Redis Client already initialized".to_string())?;
    LOCK_TTL.set(config.ttl.unwrap_or(30000)).ok();
    crate::lock::LOCK_TYPE.set(crate::lock::DistLockType::Redis).ok();
    Ok(())
}

pub(crate) async fn lock(key: &str, opt_ttl: Option<i64>, opt_wait_ms: Option<i64>) -> Result<String, String> {
    let client = REDIS_CLIENT.get().expect("Redis lock client not initialized");
    let ttl = opt_ttl.unwrap_or_else(|| *LOCK_TTL.get().unwrap_or(&30000));
    let wait_ms = opt_wait_ms.unwrap_or(30000) as u64;
    let mut conn: redis::aio::MultiplexedConnection =
        client.get_multiplexed_async_connection().await.map_err(|e| e.to_string())?;

    let val = format!("{}-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(), std::process::id());

    let start = std::time::Instant::now();
    loop {
        let result: redis::RedisResult<bool> =
            redis::cmd("SET").arg(key).arg(&val).arg("NX").arg("PX").arg(ttl).query_async(&mut conn).await;

        if let Ok(true) = result {
            return Ok(val);
        }

        if start.elapsed().as_millis() as u64 >= wait_ms {
            return Err(format!("Failed to acquire redis lock for key '{}' within {} ms", key, wait_ms));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(crate) async fn unlock(key: &str, val: &str) {
    if let Some(client) = REDIS_CLIENT.get()
        && let Ok(mut conn) = client.get_multiplexed_async_connection().await
    {
        let script = redis::Script::new(
            r#"
                if redis.call("get",KEYS[1]) == ARGV[1] then
                    return redis.call("del",KEYS[1])
                else
                    return 0
                end
                "#,
        );
        let _: redis::RedisResult<i32> = script.key(key).arg(val).invoke_async(&mut conn).await;
    }
}
