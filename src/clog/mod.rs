/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::OnceLock;
use tokio::sync::mpsc;
use tonic::transport::Channel;

use crate::grc::grc_clog::{
    log_service_client::LogServiceClient,
    LogBatchRequest, LogEntryRequest,
};
pub use crate::grc::grc_clog::log_service_server::{LogService, LogServiceServer};

pub type LogEntry = LogEntryRequest;
pub type LogBatch = LogBatchRequest;
pub use crate::grc::grc_clog::LogResponse as CLogResponse;

#[derive(Clone, Debug)]
pub struct LogContext {
    pub trace_id: String,
    pub parent_uid: Option<String>,
    pub endpoint_uid: String,
    pub service_name: String,
}

tokio::task_local! {
    pub static LOG_CTX: LogContext;
}

#[derive(Clone, Debug)]
pub struct CLogConfig {
    pub service_name: String,
    pub central_log_url: Option<String>,
    pub exclusion_routes: Vec<String>,
}

static CLOG_CONFIG: OnceLock<CLogConfig> = OnceLock::new();
static LOG_SENDER: OnceLock<mpsc::Sender<LogEntryRequest>> = OnceLock::new();

/// Initialize central logging system in rmod.
pub fn init(config: CLogConfig) {
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

pub fn get_config() -> Option<&'static CLogConfig> {
    CLOG_CONFIG.get()
}

pub fn get_current_ctx() -> Option<LogContext> {
    LOG_CTX.try_with(|ctx| ctx.clone()).ok()
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
pub fn new_log_entry(log_type: &str, action_name: &str, duration_ms: i32, status_code: i32, payload_json: String) -> Option<LogEntryRequest> {
    let ctx = get_current_ctx()?;
    let now_ms = crate::time::now_ms();
    let uid = crate::uid::new();

    Some(LogEntryRequest {
        uid,
        timestamp_unix_ms: now_ms,
        service_name: ctx.service_name,
        trace_id: ctx.trace_id,
        parent_uid: ctx.endpoint_uid,
        log_type: log_type.to_string(),
        action_name: action_name.to_string(),
        duration_ms,
        status_code,
        payload_json,
    })
}

pub fn log_db_query(sql: &str, duration_ms: i32, status_code: i32, error_msg: Option<&str>) {
    if let Some(entry) = new_log_entry(
        "DB_QUERY",
        sql,
        duration_ms,
        status_code,
        serde_json::json!({
            "sql": sql,
            "error": error_msg,
        })
        .to_string(),
    ) {
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
