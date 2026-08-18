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
use dashmap::DashMap;
use reqwest::{Client, Method, Response, header::HeaderMap};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

static DOMAIN_TIMEOUTS: LazyLock<DashMap<String, Duration>> = LazyLock::new(DashMap::new);

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .tcp_nodelay(true)
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(100)
        .build()
        .unwrap_or_default()
});

fn get_domain(url: &str) -> String {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| "default".to_string())
}

fn get_timeout(url: &str) -> Duration {
    let domain = get_domain(url);
    DOMAIN_TIMEOUTS.get(&domain).map(|t| *t).unwrap_or(Duration::from_secs(30))
}

pub fn client(url: &str, timeout: Duration) {
    let domain = get_domain(url);
    DOMAIN_TIMEOUTS.insert(domain, timeout);
}

async fn request<T: Serialize>(
    method: Method,
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    let timeout = get_timeout(url);

    let mut rb = if let (Some(q), Ok(mut u)) = (query, reqwest::Url::parse(url)) {
        u.query_pairs_mut().extend_pairs(q.iter());
        HTTP_CLIENT.request(method.clone(), u)
    } else {
        HTTP_CLIENT.request(method.clone(), url)
    };

    rb = rb.timeout(timeout);

    let ctx = clog::get_current_ctx();
    let trace_id = match ctx {
        Some(ref c) => c.trace_id.clone(),
        None => crate::uid::new(),
    };
    let parent_uid = ctx.as_ref().map(|c| c.endpoint_uid.clone());
    let endpoint_uid = crate::uid::new();
    let service_name = ctx.as_ref().map(|c| c.service_name.clone()).unwrap_or_default();
    let env_name = ctx.as_ref().map(|c| c.env_name.clone()).unwrap_or_default();

    let mut head_map = HeaderMap::new();
    if let Some(h) = headers {
        for (k, v) in h {
            if let (Ok(ref name), Ok(ref value)) =
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()), reqwest::header::HeaderValue::from_str(&v))
            {
                head_map.insert(name.clone(), value.clone());
            }
        }
    }

    if !head_map.contains_key("x-trace-id")
        && let Ok(v) = reqwest::header::HeaderValue::from_str(&trace_id)
    {
        head_map.insert("x-trace-id", v);
    }
    if let Some(ref p_uid) = parent_uid
        && !head_map.contains_key("x-parent-uid")
        && let Ok(v) = reqwest::header::HeaderValue::from_str(p_uid)
    {
        head_map.insert("x-parent-uid", v);
    }
    rb = rb.headers(head_map);

    let req_body_str = if let Some(ref b) = body { serde_json::to_string(b).unwrap_or_default() } else { String::new() };

    if let Some(b) = body {
        rb = rb.json(&b);
    }

    let action_name = format!("{}: {}", method.as_str(), url);
    let clog_config = clog::get_config();
    let is_excluded = clog_config.map(|c| c.exclusion_routes.iter().any(|r| url.contains(r) || action_name.contains(r))).unwrap_or(false);

    if !is_excluded && clog_config.is_some() {
        let req_body_val = clog::parse_body_to_json_val(&req_body_str);
        let start_payload_map = serde_json::json!({
            "endpoint": action_name,
            "url": url,
            "method": method.as_str(),
            "request_body": req_body_val,
        });

        let (pod_ip, node_name) = clog::pod_info();
        let info_map = serde_json::json!({
            "pod_ip": pod_ip,
            "node_name": node_name,
        });

        let current_user_uid = clog::get_current_ctx().and_then(|c| c.user_uid).unwrap_or_default();
        let start_now_ms = crate::time::now_ms();

        clog::push_log(clog::LogEntry {
            uid: endpoint_uid.clone(),
            timestamp_unix_ms: start_now_ms,
            env_name: env_name.clone(),
            service_name: service_name.clone(),
            trace_id: trace_id.clone(),
            parent_uid: parent_uid.clone().unwrap_or_default(),
            user_uid: current_user_uid,
            log_type: "HTTP_CALL_START".to_string(),
            action_name: action_name.clone(),
            duration_ms: 0,
            status_code: 200,
            payload_json: start_payload_map.to_string(),
            pod_name: clog::pod_name(),
            info_json: info_map.to_string(),
        });
    }

    let start_time = std::time::Instant::now();
    let res_result = rb.send().await;
    let duration_ms = start_time.elapsed().as_millis() as i32;

    match res_result {
        Ok(res) => {
            let status_code = res.status().as_u16() as i32;
            let http_res = http::Response::from(res);
            let (parts, body) = http_res.into_parts();

            let limit = std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024);

            let axum_body = axum::body::Body::new(body);
            let res_bytes = axum::body::to_bytes(axum_body, limit).await.unwrap_or_default();

            if !is_excluded && clog_config.is_some() {
                let req_body_val = clog::parse_body_to_json_val(&req_body_str);
                let res_body_lossy = String::from_utf8_lossy(&res_bytes);
                let res_body_val = clog::parse_body_to_json_val(&res_body_lossy);

                let mut payload_map = serde_json::json!({
                    "endpoint": action_name,
                    "url": url,
                    "method": method.as_str(),
                    "request_body": req_body_val,
                    "response_body": res_body_val,
                });

                if status_code >= 400 {
                    let bt = std::backtrace::Backtrace::force_capture();
                    let bt_str = format!("{}", bt);
                    let clean_st = clog::clean_stacktrace(&bt_str);
                    if !clean_st.trim().is_empty() {
                        payload_map["stacktrace"] = serde_json::Value::String(clean_st);
                    }
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
                    uid: crate::uid::new(),
                    timestamp_unix_ms: now_ms,
                    env_name,
                    service_name,
                    trace_id,
                    parent_uid: endpoint_uid,
                    user_uid: current_user_uid,
                    log_type: "HTTP_CALL_FINISH".to_string(),
                    action_name: action_name.clone(),
                    duration_ms,
                    status_code,
                    payload_json,
                    pod_name: clog::pod_name(),
                    info_json: info_map.to_string(),
                });
            }

            let reconstructed_http_res = http::Response::from_parts(parts, reqwest::Body::from(res_bytes));
            let reconstructed_res = Response::from(reconstructed_http_res);
            Ok(reconstructed_res)
        }
        Err(err) => {
            let status_code = err.status().map(|s| s.as_u16() as i32).unwrap_or(500);

            if !is_excluded && clog_config.is_some() {
                let req_body_val = clog::parse_body_to_json_val(&req_body_str);

                let bt = std::backtrace::Backtrace::force_capture();
                let bt_str = format!("{}", bt);
                let mut payload_map = serde_json::json!({
                    "endpoint": action_name,
                    "url": url,
                    "method": method.as_str(),
                    "request_body": req_body_val,
                    "error": err.to_string(),
                });

                let clean_st = clog::clean_stacktrace(&bt_str);
                if !clean_st.trim().is_empty() {
                    payload_map["stacktrace"] = serde_json::Value::String(clean_st);
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
                    uid: crate::uid::new(),
                    timestamp_unix_ms: now_ms,
                    env_name,
                    service_name,
                    trace_id,
                    parent_uid: endpoint_uid,
                    user_uid: current_user_uid,
                    log_type: "HTTP_CALL_FINISH".to_string(),
                    action_name: action_name.clone(),
                    duration_ms,
                    status_code,
                    payload_json,
                    pod_name: clog::pod_name(),
                    info_json: info_map.to_string(),
                });
            }

            Err(err)
        }
    }
}

pub async fn get(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
) -> Result<Response, reqwest::Error> {
    request(Method::GET, url, headers, query, None::<()>).await
}

pub async fn post<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    request(Method::POST, url, headers, query, body).await
}

pub async fn put<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    request(Method::PUT, url, headers, query, body).await
}

pub async fn patch<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    request(Method::PATCH, url, headers, query, body).await
}

pub async fn delete(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
) -> Result<Response, reqwest::Error> {
    request(Method::DELETE, url, headers, query, None::<()>).await
}

#[cfg(test)]
pub(crate) fn clear_cache() {
    DOMAIN_TIMEOUTS.clear();
}
