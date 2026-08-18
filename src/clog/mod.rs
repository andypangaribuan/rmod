/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

mod info;

use std::sync::OnceLock;
use tokio::sync::mpsc;
use tonic::transport::Channel;

pub use crate::grc::grc_clog::log_service_server::{LogService, LogServiceServer};
use crate::grc::grc_clog::{LogBatchRequest, LogEntryRequest, log_service_client::LogServiceClient};
pub use info::*;

pub type LogEntry = LogEntryRequest;
pub type LogBatch = LogBatchRequest;
pub use crate::grc::grc_clog::LogResponse as CLogResponse;

#[derive(Clone, Debug)]
pub struct Context {
    pub trace_id: String,
    pub parent_uid: Option<String>,
    pub user_uid: Option<String>,
    pub endpoint_uid: String,
    pub service_name: String,
    pub env_name: String,
}

tokio::task_local! {
    pub static LOG_CTX: std::cell::RefCell<Context>;
}

#[derive(Clone, Debug)]
pub struct Config {
    pub service_name: String,
    pub central_log_url: Option<String>,
    pub exclusion_routes: Vec<String>,
    pub environment: String,
}

static CLOG_CONFIG: OnceLock<Config> = OnceLock::new();
static LOG_SENDER: OnceLock<mpsc::Sender<LogEntryRequest>> = OnceLock::new();

/// Initialize central logging system in rmod.
pub fn init(config: Config) {
    let service_name = config.service_name.clone();
    let url = config.central_log_url.clone();
    let _ = CLOG_CONFIG.set(config);

    if let Some(url_str) = url
        && !url_str.trim().is_empty()
    {
        let (tx, rx) = mpsc::channel::<LogEntryRequest>(10_000);
        let _ = LOG_SENDER.set(tx);
        tokio::spawn(worker_loop(rx, url_str, service_name));
    }
}

pub fn get_config() -> Option<&'static Config> {
    CLOG_CONFIG.get()
}

pub fn get_current_ctx() -> Option<Context> {
    LOG_CTX.try_with(|ctx| ctx.borrow().clone()).ok()
}

/// Set the user_uid for the active request context.
pub fn set_user_uid(user_uid: impl Into<String>) {
    let uid = user_uid.into();
    let _ = LOG_CTX.try_with(|ctx| {
        ctx.borrow_mut().user_uid = Some(uid);
    });
}

/// Push a log entry asynchronously into the background buffer.
pub fn push_log(entry: LogEntryRequest) {
    if let Some(sender) = LOG_SENDER.get() {
        match sender.try_send(entry) {
            Ok(_) => {}
            Err(mpsc::error::TrySendError::Full(dropped_entry)) => {
                // Drop policy: Drop non-ERROR logs when buffer is full under extreme backpressure
                if dropped_entry.log_type == "ERROR" {
                    eprintln!(
                        "[clog][BUFFER_FULL_EMERGENCY] [{}] trace={} action={}",
                        dropped_entry.service_name, dropped_entry.trace_id, dropped_entry.action_name
                    );
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }
}

/// Helper function to create a new LogEntry with context automatically populated.
pub fn new_log_entry(
    log_type: &str,
    action_name: &str,
    duration_ms: i32,
    status_code: i32,
    payload_json: String,
) -> Option<LogEntryRequest> {
    let clog_config = get_config()?;
    let is_excluded = clog_config.exclusion_routes.iter().any(|r| action_name.contains(r));
    if is_excluded {
        return None;
    }

    let ctx = get_current_ctx();
    let service_name = ctx.as_ref().map(|c| c.service_name.clone()).unwrap_or_else(|| clog_config.service_name.clone());
    let env_name = ctx.as_ref().map(|c| c.env_name.clone()).unwrap_or_else(|| clog_config.environment.clone());
    let trace_id = ctx.as_ref().map(|c| c.trace_id.clone()).unwrap_or_default();
    let parent_uid = ctx.as_ref().map(|c| c.endpoint_uid.clone()).unwrap_or_default();
    let user_uid = ctx.as_ref().and_then(|c| c.user_uid.clone()).unwrap_or_default();

    let now_ms = crate::time::now_ms();
    let uid = crate::uid::new();

    let (pod_ip, node_name) = pod_info();
    let info_map = serde_json::json!({
        "pod_ip": pod_ip,
        "node_name": node_name,
    });

    Some(LogEntryRequest {
        uid,
        timestamp_unix_ms: now_ms,
        env_name,
        service_name,
        trace_id,
        parent_uid,
        user_uid,
        log_type: log_type.to_string(),
        action_name: action_name.to_string(),
        duration_ms,
        status_code,
        payload_json,
        pod_name: pod_name(),
        info_json: info_map.to_string(),
    })
}

/// Developer custom logging helper to log any serializable payload to central log.
pub fn custom_log<T: serde::Serialize>(log_type: &str, action_name: &str, payload: T) {
    let payload_json = serde_json::to_string(&payload).unwrap_or_default();
    if let Some(entry) = new_log_entry(log_type, action_name, 0, 200, payload_json) {
        push_log(entry);
    }
}

pub fn log<T: serde::Serialize>(log_type: &str, action_name: &str, payload: T) {
    custom_log(log_type, action_name, payload);
}

pub fn info<T: serde::Serialize>(action_name: &str, payload: T) {
    custom_log("INFO", action_name, payload);
}

pub fn warn<T: serde::Serialize>(action_name: &str, payload: T) {
    custom_log("WARN", action_name, payload);
}

pub fn error<T: serde::Serialize>(action_name: &str, payload: T) {
    custom_log("ERROR", action_name, payload);
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_query(
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_QUERY", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_tx_query(
    tx_id: &str,
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_QUERY", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

pub fn log_tx_begin(tx_id: &str, key: Option<&str>, duration_ms: i32, status_code: i32, error_msg: Option<&str>, stacktrace: Option<&str>) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_BEGIN", "BEGIN", duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

pub fn log_tx_commit(
    tx_id: &str,
    key: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_COMMIT", "COMMIT", duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

pub fn log_tx_rollback(
    tx_id: &str,
    key: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_ROLLBACK", "ROLLBACK", duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_update(
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_UPDATE", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_tx_update(
    tx_id: &str,
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_UPDATE", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_execute(
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_EXEC", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn log_db_tx_execute(
    tx_id: &str,
    key: Option<&str>,
    sql: &str,
    args: Option<&[String]>,
    response: Option<&str>,
    duration_ms: i32,
    status_code: i32,
    error_msg: Option<&str>,
    stacktrace: Option<&str>,
) {
    let db_conn = crate::store::get_db_conn_info(key);
    let mut payload = serde_json::json!({
        "tx_id": tx_id,
        "db_conn": db_conn,
        "sql": sql,
        "args": args,
        "response": response,
        "error": error_msg,
    });
    if let Some(st) = stacktrace {
        payload["stacktrace"] = serde_json::Value::String(st.to_string());
    }
    if let Some(entry) = new_log_entry("DB_TX_EXEC", sql, duration_ms, status_code, payload.to_string()) {
        push_log(entry);
    }
}

pub fn log_grpc_call(action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) {
    if let Some(entry) = new_log_entry("GRPC_CALL", action_name, duration_ms, status_code, payload_json) {
        push_log(entry);
    }
}

pub fn log_dist_lock_pg_lock(action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) {
    if let Some(entry) = new_log_entry("DIST_LOCK_PG_LOCK", action_name, duration_ms, status_code, payload_json) {
        push_log(entry);
    }
}

pub fn log_dist_lock_pg_unlock(action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) {
    if let Some(entry) = new_log_entry("DIST_LOCK_PG_UNLOCK", action_name, duration_ms, status_code, payload_json) {
        push_log(entry);
    }
}

pub fn log_dist_lock_redis_lock(action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) {
    if let Some(entry) = new_log_entry("DIST_LOCK_REDIS_LOCK", action_name, duration_ms, status_code, payload_json) {
        push_log(entry);
    }
}

pub fn log_dist_lock_redis_unlock(action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) {
    if let Some(entry) = new_log_entry("DIST_LOCK_REDIS_UNLOCK", action_name, duration_ms, status_code, payload_json) {
        push_log(entry);
    }
}

/// Background worker loop that buffers logs and flushes batches to central-log gRPC server.
async fn worker_loop(mut rx: mpsc::Receiver<LogEntryRequest>, target_url: String, service_name: String) {
    let mut buffer: Vec<LogEntryRequest> = Vec::with_capacity(500);
    let mut total_bytes: usize = 0;
    let max_batch_size = 500;
    let max_batch_bytes = 2 * 1024 * 1024; // 2MB
    let flush_interval = tokio::time::Duration::from_millis(500);

    let mut shutdown_rx = crate::util::lifecycle::subscribe();
    let mut client: Option<LogServiceClient<Channel>> = None;

    loop {
        let timeout_future = tokio::time::sleep(flush_interval);
        tokio::pin!(timeout_future);
        tokio::select! {
            maybe_entry = rx.recv() => {
                match maybe_entry {
                    Some(entry) => {
                        total_bytes += entry.payload_json.len();
                        buffer.push(entry);

                        if buffer.len() >= max_batch_size || total_bytes >= max_batch_bytes {
                            flush_batch(&mut client, &target_url, &service_name, &mut buffer, &mut total_bytes).await;
                        }
                    }
                    None => {
                        // Channel closed, flush remaining and exit loop
                        if !buffer.is_empty() {
                            flush_batch(&mut client, &target_url, &service_name, &mut buffer, &mut total_bytes).await;
                        }
                        break;
                    }
                }
            }
            _ = &mut timeout_future => {
                if !buffer.is_empty() {
                    flush_batch(&mut client, &target_url, &service_name, &mut buffer, &mut total_bytes).await;
                }
            }
            _ = shutdown_rx.recv() => {
                // Application shutdown triggered: drain remaining logs and flush batch
                while let Ok(entry) = rx.try_recv() {
                    total_bytes += entry.payload_json.len();
                    buffer.push(entry);
                }
                if !buffer.is_empty() {
                    flush_batch(&mut client, &target_url, &service_name, &mut buffer, &mut total_bytes).await;
                }
                break;
            }
        }
    }
}

async fn flush_batch(
    client: &mut Option<LogServiceClient<Channel>>,
    target_url: &str,
    service_name: &str,
    buffer: &mut Vec<LogEntryRequest>,
    total_bytes: &mut usize,
) {
    if buffer.is_empty() {
        return;
    }

    let entries = std::mem::take(buffer);
    *total_bytes = 0;

    // Connect if client is not connected
    if client.is_none() {
        if target_url.starts_with("https://") {
            let _ = crate::rustls::crypto::ring::default_provider().install_default();
        }

        match crate::util::grpc_client::connect(target_url).await {
            Ok(channel) => *client = Some(LogServiceClient::new(channel)),
            Err(e) => {
                eprintln!("[clog][WARN] Failed to connect to central-log service '{}': {}", target_url, e);
                return;
            }
        }
    }

    if let Some(c) = client {
        let req = tonic::Request::new(LogBatchRequest { entries });
        if let Err(e) = c.push_batch(req).await {
            eprintln!("[clog][WARN] Failed to push log batch for service '{}': {}", service_name, e);
            // Reset client to trigger reconnect on next attempt
            *client = None;
        }
    }
}

pub(crate) fn parse_body_to_json_val(s: &str) -> serde_json::Value {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return serde_json::Value::Null;
    }

    if ((trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed)
    {
        return v;
    }

    if s.len() > 100_000 {
        serde_json::Value::String(format!("{}... [TRUNCATED]", &s[..100_000]))
    } else {
        serde_json::Value::String(s.to_string())
    }
}
