/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::clog;
use futures_util::future::BoxFuture;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{MissedTickBehavior, interval};

struct Job {
    name: String,
    duration: Duration,
    handler: fn() -> BoxFuture<'static, ()>,
    is_every: bool,
    zero_start: bool,
}

static JOBS: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

fn get_jobs() -> &'static Mutex<Vec<Job>> {
    JOBS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn add(name: &str, duration: &str, handler: fn() -> BoxFuture<'static, ()>, is_every: bool, zero_start: bool) {
    let mut jobs = get_jobs().lock().unwrap();
    let duration = crate::time::to_duration(duration);
    jobs.push(Job { name: name.to_string(), duration, handler, is_every, zero_start });
}

async fn run_job_handler(name: String, handler: fn() -> BoxFuture<'static, ()>) {
    let trace_id = crate::uid::new();
    let endpoint_uid = crate::uid::new();

    let clog_config = clog::get_config();
    let service_name = clog_config.map(|c| c.service_name.clone()).unwrap_or_default();
    let env_name = clog_config.map(|c| c.environment.clone()).unwrap_or_default();
    let is_excluded = clog_config.map(|c| c.exclusion_routes.iter().any(|r| name.contains(r))).unwrap_or(false);

    let log_ctx = clog::Context {
        trace_id: trace_id.clone(),
        parent_uid: None,
        user_uid: None,
        endpoint_uid: endpoint_uid.clone(),
        service_name: service_name.clone(),
        env_name: env_name.clone(),
    };

    let start_time = std::time::Instant::now();
    let join_res = clog::LOG_CTX.scope(std::cell::RefCell::new(log_ctx), tokio::spawn((handler)())).await;
    let duration_ms = start_time.elapsed().as_millis() as i32;

    let (status_code, error_msg, stacktrace) = match join_res {
        Ok(()) => (200, None, None),
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let clean_st = clog::clean_stacktrace(&bt_str);
            let st = if !clean_st.trim().is_empty() { Some(clean_st) } else { None };
            if e.is_panic() {
                tracing::error!("Background job '{}' panicked: {:?}", name, e);
                (500, Some("Job panicked".to_string()), st)
            } else {
                (500, Some(e.to_string()), st)
            }
        }
    };

    if !is_excluded && clog_config.is_some() {
        let mut payload_map = serde_json::json!({
            "job_name": name,
            "error": error_msg,
        });

        if let Some(st) = stacktrace {
            payload_map["stacktrace"] = serde_json::Value::String(st);
        }

        let (pod_ip, node_name) = clog::pod_info();
        let info_map = serde_json::json!({
            "pod_ip": pod_ip,
            "node_name": node_name,
        });

        let payload_json = payload_map.to_string();
        let current_user_uid = clog::get_current_ctx().and_then(|c| c.user_uid).unwrap_or_default();
        let now_ms = crate::time::now_ms();

        clog::push_log(clog::LogEntry {
            uid: endpoint_uid,
            timestamp_unix_ms: now_ms,
            env_name,
            service_name,
            trace_id,
            parent_uid: String::new(),
            log_type: "JOB_EXECUTION".to_string(),
            user_uid: current_user_uid,
            action_name: name,
            duration_ms,
            status_code,
            payload_json,
            pod_name: clog::pod_name(),
            info_json: info_map.to_string(),
        });
    }
}

pub fn start() {
    let mut jobs_lock = get_jobs().lock().unwrap();
    let jobs = std::mem::take(&mut *jobs_lock);

    for job in jobs {
        tokio::spawn(async move {
            let mut shutdown_rx = crate::util::lifecycle::subscribe();

            if job.zero_start {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
                let now_ns = now.as_nanos();
                let minute_ns = 60_000_000_000u128;
                let next_ns = ((now_ns / minute_ns) + 1) * minute_ns;
                let delay_ns = (next_ns - now_ns) as u64;

                tokio::select! {
                    _ = shutdown_rx.recv() => return,
                    _ = tokio::time::sleep(Duration::from_nanos(delay_ns)) => {}
                }
            }

            if job.is_every {
                let mut interval = interval(job.duration);
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => break,
                        _ = interval.tick() => {
                            run_job_handler(job.name.clone(), job.handler).await;
                        }
                    }
                }
            } else {
                loop {
                    run_job_handler(job.name.clone(), job.handler).await;

                    tokio::select! {
                        _ = shutdown_rx.recv() => break,
                        _ = tokio::time::sleep(job.duration) => {}
                    }
                }
            }
        });
    }
}
