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
use http_body_util::BodyExt;
pub use prost;
use std::net::SocketAddr;
pub use tonic;
use tonic::transport::Server;
pub use tonic_health;

#[derive(Clone)]
pub struct ClogGrpcService<S> {
    inner: S,
}

impl<S: tonic::server::NamedService> tonic::server::NamedService for ClogGrpcService<S> {
    const NAME: &'static str = S::NAME;
}

impl<S> tonic::codegen::Service<tonic::codegen::http::Request<tonic::body::BoxBody>> for ClogGrpcService<S>
where
    S: tonic::codegen::Service<
            tonic::codegen::http::Request<tonic::body::BoxBody>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
    type Error = std::convert::Infallible;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: tonic::codegen::http::Request<tonic::body::BoxBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let path = parts.uri.path().to_string();

            let is_health_check = path.starts_with("/grpc.health.v1.Health");

            let trace_id =
                parts.headers.get("x-trace-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(crate::uid::new);

            let parent_uid = parts.headers.get("x-parent-uid").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let user_uid = parts.headers.get("x-user-uid").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let partner_uid = parts.headers.get("x-partner-uid").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

            let endpoint_uid = crate::uid::new();

            let clog_config = clog::get_config();
            let service_name = clog_config.map(|c| c.service_name.clone()).unwrap_or_default();
            let env_name = clog_config.map(|c| c.environment.clone()).unwrap_or_default();

            let is_excluded =
                is_health_check || clog_config.map(|c| c.exclusion_routes.iter().any(|r| path.starts_with(r))).unwrap_or(false);

            let log_ctx = clog::Context {
                trace_id: trace_id.clone(),
                parent_uid: parent_uid.clone(),
                user_uid: user_uid.clone(),
                partner_uid: partner_uid.clone(),
                endpoint_uid: endpoint_uid.clone(),
                service_name: service_name.clone(),
                env_name: env_name.clone(),
            };

            let start_time = std::time::Instant::now();
            let limit = std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024);

            let req_axum_body = axum::body::Body::new(body);
            let req_bytes = axum::body::to_bytes(req_axum_body, limit).await.unwrap_or_default();
            let req_body_box =
                tonic::body::BoxBody::new(http_body_util::Full::new(req_bytes.clone()).map_err(|_| tonic::Status::internal("body error")));
            let req_reconstructed = tonic::codegen::http::Request::from_parts(parts, req_body_box);

            if !is_excluded && clog_config.is_some() {
                let req_json = decode_grpc_body_to_json(&req_bytes, &path, true);
                let payload_map = serde_json::json!({
                    "endpoint": path,
                    "path": path,
                    "request_body": req_json,
                });

                let (pod_ip, node_name) = clog::pod_info();
                let info_map = serde_json::json!({
                    "pod_ip": pod_ip,
                    "node_name": node_name,
                });

                let start_now_us = crate::time::now_us();
                clog::push_log(clog::LogEntry {
                    uid: endpoint_uid.clone(),
                    timestamp_unix_us: start_now_us,
                    env_name: env_name.clone(),
                    service_name: service_name.clone(),
                    trace_id: trace_id.clone(),
                    parent_uid: parent_uid.clone().unwrap_or_default(),
                    user_uid: user_uid.clone().unwrap_or_default(),
                    partner_uid: partner_uid.clone().unwrap_or_default(),
                    log_type: "GRPC_INCOMING".to_string(),
                    action_name: path.clone(),
                    duration_ms: 0,
                    status_code: 0,
                    payload_json: payload_map.to_string(),
                    pod_name: clog::pod_name(),
                    info_json: info_map.to_string(),
                });
            }

            use tower::ServiceExt;
            match inner.ready().await {
                Ok(ready_svc) => {
                    clog::LOG_CTX
                        .scope(std::cell::RefCell::new(log_ctx), async move {
                            let res_result = ready_svc.call(req_reconstructed).await;
                            match res_result {
                                Ok(response) => {
                                    let (res_parts, res_body) = response.into_parts();
                                    let duration_ms = start_time.elapsed().as_millis() as i32;

                                    let grpc_status = res_parts.headers.get("grpc-status").and_then(|v| v.to_str().ok()).unwrap_or("0").to_string();

                                    let status_code = if grpc_status == "0" || res_parts.status.is_success() { 200 } else { 500 };

                                    let res_axum_body = axum::body::Body::new(res_body);
                                    let res_bytes = axum::body::to_bytes(res_axum_body, limit).await.unwrap_or_default();

                                    if !is_excluded && clog_config.is_some() {
                                        let res_json = decode_grpc_body_to_json(&res_bytes, &path, false);

                                        let mut payload_map = serde_json::json!({
                                            "endpoint": path,
                                            "path": path,
                                            "response_body": res_json,
                                            "grpc_status": grpc_status,
                                        });

                                        let (pod_ip, node_name) = clog::pod_info();
                                        let info_map = serde_json::json!({
                                            "pod_ip": pod_ip,
                                            "node_name": node_name,
                                        });

                                        if status_code != 200 {
                                            let bt = std::backtrace::Backtrace::force_capture();
                                            let bt_str = format!("{}", bt);
                                            let clean_st = clog::clean_stacktrace(&bt_str);
                                            if !clean_st.trim().is_empty() {
                                                payload_map["stacktrace"] = serde_json::Value::String(clean_st);
                                            }
                                        }

                                        let current_user_uid = clog::get_current_ctx().and_then(|c| c.user_uid).unwrap_or_default();
                                        let current_partner_uid = clog::get_current_ctx().and_then(|c| c.partner_uid).unwrap_or_default();
                                        let finish_now_us = crate::time::now_us();
                                        clog::push_log(clog::LogEntry {
                                            uid: crate::uid::new(),
                                            timestamp_unix_us: finish_now_us,
                                            env_name,
                                            service_name,
                                            trace_id,
                                            parent_uid: endpoint_uid,
                                            user_uid: current_user_uid,
                                            partner_uid: current_partner_uid,
                                            log_type: "GRPC_RESPONSE".to_string(),
                                            action_name: path.clone(),
                                            duration_ms,
                                            status_code,
                                            payload_json: payload_map.to_string(),
                                            pod_name: clog::pod_name(),
                                            info_json: info_map.to_string(),
                                        });
                                    }

                                    let res_body_box =
                                        tonic::body::BoxBody::new(http_body_util::Full::new(res_bytes).map_err(|_| tonic::Status::internal("body error")));
                                    let res_reconstructed = tonic::codegen::http::Response::from_parts(res_parts, res_body_box);
                                    Ok(res_reconstructed)
                                }
                                Err(err) => Err(err),
                            }
                        })
                        .await
                }
                Err(err) => Err(err),
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct ClogGrpcClientService<S> {
    inner: S,
}

pub fn grpc_client<S>(service: S) -> ClogGrpcClientService<S> {
    ClogGrpcClientService { inner: service }
}

impl<S> tonic::codegen::Service<tonic::codegen::http::Request<tonic::body::BoxBody>> for ClogGrpcClientService<S>
where
    S: tonic::codegen::Service<
            tonic::codegen::http::Request<tonic::body::BoxBody>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
        > + Clone
        + Send
        + 'static,
    S::Error: Into<tonic::codegen::StdError> + std::fmt::Display + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: tonic::codegen::http::Request<tonic::body::BoxBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let ctx = clog::get_current_ctx();
            let trace_id = match ctx {
                Some(ref c) => c.trace_id.clone(),
                None => crate::uid::new(),
            };
            let parent_uid = ctx.as_ref().map(|c| c.endpoint_uid.clone());
            let endpoint_uid = crate::uid::new();
            let service_name = ctx.as_ref().map(|c| c.service_name.clone()).unwrap_or_default();
            let env_name = ctx.as_ref().map(|c| c.env_name.clone()).unwrap_or_default();

            if let Ok(v) = tonic::codegen::http::HeaderValue::from_str(&trace_id) {
                req.headers_mut().insert("x-trace-id", v);
            }
            if let Some(ref p_uid) = parent_uid
                && let Ok(v) = tonic::codegen::http::HeaderValue::from_str(p_uid)
            {
                req.headers_mut().insert("x-parent-uid", v);
            }
            if let Some(u_uid) = ctx.as_ref().and_then(|c| c.user_uid.as_ref())
                && let Ok(v) = tonic::codegen::http::HeaderValue::from_str(u_uid)
            {
                req.headers_mut().insert("x-user-uid", v);
            }
            if let Some(pt_uid) = ctx.as_ref().and_then(|c| c.partner_uid.as_ref())
                && let Ok(v) = tonic::codegen::http::HeaderValue::from_str(pt_uid)
            {
                req.headers_mut().insert("x-partner-uid", v);
            }

            let (parts, body) = req.into_parts();
            let path = parts.uri.path().to_string();

            let is_health_check = path.starts_with("/grpc.health.v1.Health");
            let clog_config = clog::get_config();
            let is_excluded =
                is_health_check || clog_config.map(|c| c.exclusion_routes.iter().any(|r| path.starts_with(r))).unwrap_or(false);

            let start_time = std::time::Instant::now();
            let limit = std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024);

            let req_axum_body = axum::body::Body::new(body);
            let req_bytes = axum::body::to_bytes(req_axum_body, limit).await.unwrap_or_default();
            let req_body_box =
                tonic::body::BoxBody::new(http_body_util::Full::new(req_bytes.clone()).map_err(|_| tonic::Status::internal("body error")));
            let req_reconstructed = tonic::codegen::http::Request::from_parts(parts, req_body_box);

            use tower::ServiceExt;
            let res_result = match inner.ready().await {
                Ok(ready_svc) => ready_svc.call(req_reconstructed).await,
                Err(err) => {
                    let duration_ms = start_time.elapsed().as_millis() as i32;
                    if !is_excluded && clog_config.is_some() {
                        let bt = std::backtrace::Backtrace::force_capture();
                        let bt_str = format!("{}", bt);
                        let mut payload_map = serde_json::json!({
                            "endpoint": path,
                            "path": path,
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

                        let finish_now_us = crate::time::now_us();
                        let current_user_uid = clog::get_current_ctx().and_then(|c| c.user_uid).unwrap_or_default();
                        let current_partner_uid = clog::get_current_ctx().and_then(|c| c.partner_uid).unwrap_or_default();

                        clog::push_log(clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_us: finish_now_us,
                            env_name,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            user_uid: current_user_uid,
                            partner_uid: current_partner_uid,
                            log_type: "GRPC_CALL".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code: 500,
                            payload_json: payload_map.to_string(),
                            pod_name: clog::pod_name(),
                            info_json: info_map.to_string(),
                        });
                    }
                    return Err(err);
                }
            };

            match res_result {
                Ok(response) => {
                    let (res_parts, res_body) = response.into_parts();
                    let duration_ms = start_time.elapsed().as_millis() as i32;

                    let grpc_status = res_parts.headers.get("grpc-status").and_then(|v| v.to_str().ok()).unwrap_or("0").to_string();

                    let status_code = if grpc_status == "0" || res_parts.status.is_success() { 200 } else { 500 };

                    let res_axum_body = axum::body::Body::new(res_body);
                    let res_bytes = axum::body::to_bytes(res_axum_body, limit).await.unwrap_or_default();

                    if !is_excluded && clog_config.is_some() {
                        let req_json = decode_grpc_body_to_json(&req_bytes, &path, true);
                        let res_json = decode_grpc_body_to_json(&res_bytes, &path, false);

                        let mut payload_map = serde_json::json!({
                            "endpoint": path,
                            "path": path,
                            "request_body": req_json,
                            "response_body": res_json,
                            "grpc_status": grpc_status,
                        });

                        if status_code != 200 {
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
                        let current_partner_uid = clog::get_current_ctx().and_then(|c| c.partner_uid).unwrap_or_default();
                        let finish_now_us = crate::time::now_us();

                        clog::push_log(clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_us: finish_now_us,
                            env_name,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            user_uid: current_user_uid,
                            partner_uid: current_partner_uid,
                            log_type: "GRPC_CALL".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code,
                            payload_json,
                            pod_name: clog::pod_name(),
                            info_json: info_map.to_string(),
                        });
                    }

                    let res_body_box =
                        tonic::body::BoxBody::new(http_body_util::Full::new(res_bytes).map_err(|_| tonic::Status::internal("body error")));
                    let res_reconstructed = tonic::codegen::http::Response::from_parts(res_parts, res_body_box);
                    Ok(res_reconstructed)
                }
                Err(err) => {
                    let duration_ms = start_time.elapsed().as_millis() as i32;
                    if !is_excluded && clog_config.is_some() {
                        let bt = std::backtrace::Backtrace::force_capture();
                        let bt_str = format!("{}", bt);
                        let mut payload_map = serde_json::json!({
                            "endpoint": path,
                            "path": path,
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

                        let finish_now_us = crate::time::now_us();
                        let current_user_uid = clog::get_current_ctx().and_then(|c| c.user_uid).unwrap_or_default();
                        let current_partner_uid = clog::get_current_ctx().and_then(|c| c.partner_uid).unwrap_or_default();

                        clog::push_log(clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_us: finish_now_us,
                            env_name,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            user_uid: current_user_uid,
                            partner_uid: current_partner_uid,
                            log_type: "GRPC_CALL".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code: 500,
                            payload_json: payload_map.to_string(),
                            pod_name: clog::pod_name(),
                            info_json: info_map.to_string(),
                        });
                    }
                    Err(err)
                }
            }
        })
    }
}

pub async fn grpc<S, F>(addr: &str, service: S, on_start: Option<F>)
where
    S: tonic::codegen::Service<
            tonic::codegen::http::Request<tonic::body::BoxBody>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    F: FnOnce(),
{
    let addr: SocketAddr = addr.parse().unwrap_or_else(|e| {
        tracing::error!("Failed to parse gRPC bind address '{}': {}", addr, e);
        std::process::exit(1);
    });
    crate::util::lifecycle::start();
    let mut shutdown_rx = crate::util::lifecycle::subscribe();

    if let Some(f) = on_start {
        f();
    }

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<S>().await;
    health_reporter.set_service_status("", tonic_health::ServingStatus::Serving).await;

    let wrapped_service = ClogGrpcService { inner: service.clone() };

    if let Err(e) = Server::builder()
        .add_service(health_service)
        .add_service(wrapped_service)
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
    {
        tracing::error!("gRPC server failed: {}", e);
        std::process::exit(1);
    }

    crate::util::lifecycle::wait().await;
}

use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

pub type ProtoFieldRegistry = HashMap<String, HashMap<u32, String>>;
pub static PROTO_REGISTRY: OnceLock<ProtoFieldRegistry> = OnceLock::new();

pub fn load_proto_registry() -> &'static ProtoFieldRegistry {
    PROTO_REGISTRY.get_or_init(|| {
        let mut registry = HashMap::new();
        let search_dirs = ["proto", "res/proto", "../res/proto", "src/proto", "res"];
        for dir in search_dirs {
            let path = std::path::Path::new(dir);
            if path.exists() {
                scan_dir_for_protos(path, &mut registry);
            }
        }
        registry
    })
}

fn scan_dir_for_protos(dir: &std::path::Path, registry: &mut ProtoFieldRegistry) {
    if dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                scan_dir_for_protos(&p, registry);
            } else if p.extension().and_then(|s| s.to_str()) == Some("proto")
                && let Ok(content) = std::fs::read_to_string(&p)
            {
                parse_proto_content(&content, registry);
            }
        }
    }
}

fn parse_proto_content(content: &str, registry: &mut ProtoFieldRegistry) {
    let mut current_message: Option<String> = None;
    let mut service_methods: HashMap<String, (String, String)> = HashMap::new();

    for raw_line in content.lines() {
        let line = if let Some((before_comment, _)) = raw_line.split_once("//") { before_comment.trim() } else { raw_line.trim() };
        if line.is_empty() {
            continue;
        }

        if line.starts_with("rpc ") && line.contains("returns") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let rpc_name = parts[1].trim();
                let req_type = parts[2].trim_matches(|c| c == '(' || c == ')').to_string();
                let resp_type = parts[4].trim_matches(|c| c == '(' || c == ')' || c == '{' || c == '}').to_string();
                service_methods.insert(rpc_name.to_string(), (req_type, resp_type));
            }
        }

        if line.starts_with("message ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let msg_name = parts[1].trim_matches('{').to_string();
                current_message = Some(msg_name);
            }
        } else if line == "}" {
            current_message = None;
        } else if let Some(ref msg_name) = current_message
            && line.contains('=')
            && line.ends_with(';')
        {
            let clean_line = line.trim_matches(';');
            let parts: Vec<&str> = clean_line.split('=').collect();
            if parts.len() == 2 {
                let left_parts: Vec<&str> = parts[0].split_whitespace().collect();
                let right_tag = parts[1].trim();
                if let (Some(field_name), Ok(tag)) = (left_parts.last(), right_tag.parse::<u32>()) {
                    let field_map = registry.entry(msg_name.clone()).or_default();
                    field_map.insert(tag, field_name.to_string());
                }
            }
        }
    }

    for (rpc_name, (req_type, resp_type)) in service_methods {
        if let Some(req_fields) = registry.get(&req_type).cloned() {
            registry.insert(format!("{}:req", rpc_name), req_fields);
        }
        if let Some(resp_fields) = registry.get(&resp_type).cloned() {
            registry.insert(format!("{}:res", rpc_name), resp_fields);
        }
    }
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift = 0;
    while *pos < buf.len() {
        let byte = buf[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

pub fn decode_grpc_body_to_json(raw_bytes: &[u8], path: &str, is_request: bool) -> Value {
    if raw_bytes.is_empty() {
        return serde_json::json!({});
    }

    let payload = if raw_bytes.len() >= 5 {
        let msg_len = u32::from_be_bytes([raw_bytes[1], raw_bytes[2], raw_bytes[3], raw_bytes[4]]) as usize;
        if raw_bytes.len() >= 5 + msg_len { &raw_bytes[5..5 + msg_len] } else { &raw_bytes[5..] }
    } else {
        raw_bytes
    };

    if payload.is_empty() {
        return serde_json::json!({});
    }

    let registry = load_proto_registry();
    let method_name = path.rsplit('/').next().unwrap_or(path);

    let key = if is_request { format!("{}:req", method_name) } else { format!("{}:res", method_name) };

    let field_map = registry.get(&key).or_else(|| {
        if is_request { registry.get(&format!("{}Request", method_name)) } else { registry.get(&format!("{}Response", method_name)) }
    });

    decode_protobuf_wire(payload, field_map)
}

fn decode_protobuf_wire(payload: &[u8], field_map: Option<&HashMap<u32, String>>) -> Value {
    let mut map = Map::new();
    let mut pos = 0;

    while pos < payload.len() {
        let key = match read_varint(payload, &mut pos) {
            Some(k) => k,
            None => break,
        };

        let tag = (key >> 3) as u32;
        let wire_type = (key & 0x07) as u8;

        if tag == 0 {
            break;
        }

        let field_key = field_map.and_then(|m| m.get(&tag).cloned()).unwrap_or_else(|| tag.to_string());

        let val = match wire_type {
            0 => match read_varint(payload, &mut pos) {
                Some(v) => Value::from(v),
                None => break,
            },
            1 => {
                if pos + 8 > payload.len() {
                    break;
                }
                let bytes: [u8; 8] = payload[pos..pos + 8].try_into().unwrap();
                pos += 8;
                let f_val = f64::from_le_bytes(bytes);
                if f_val.is_finite() { Value::from(f_val) } else { Value::from(u64::from_le_bytes(bytes)) }
            }
            2 => {
                let len = match read_varint(payload, &mut pos) {
                    Some(l) => l as usize,
                    None => break,
                };
                if pos + len > payload.len() {
                    break;
                }
                let sub_bytes = &payload[pos..pos + len];
                pos += len;

                if let Ok(s) = std::str::from_utf8(sub_bytes) {
                    if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t') {
                        Value::String(s.to_string())
                    } else {
                        let sub_json = decode_protobuf_wire(sub_bytes, None);
                        if sub_json.is_object() && !sub_json.as_object().unwrap().is_empty() {
                            sub_json
                        } else {
                            Value::String(s.to_string())
                        }
                    }
                } else {
                    let sub_json = decode_protobuf_wire(sub_bytes, None);
                    if sub_json.is_object() && !sub_json.as_object().unwrap().is_empty() {
                        sub_json
                    } else {
                        Value::String(String::from_utf8_lossy(sub_bytes).to_string())
                    }
                }
            }
            5 => {
                if pos + 4 > payload.len() {
                    break;
                }
                let bytes: [u8; 4] = payload[pos..pos + 4].try_into().unwrap();
                pos += 4;
                let f_val = f32::from_le_bytes(bytes);
                if f_val.is_finite() { Value::from(f_val as f64) } else { Value::from(u32::from_le_bytes(bytes)) }
            }
            _ => break,
        };

        if let Some(existing) = map.get_mut(&field_key) {
            if let Value::Array(arr) = existing {
                arr.push(val);
            } else {
                let prev = map.remove(&field_key).unwrap();
                map.insert(field_key.clone(), Value::Array(vec![prev, val]));
            }
        } else {
            map.insert(field_key, val);
        }
    }

    if map.is_empty() { Value::String(String::from_utf8_lossy(payload).to_string()) } else { Value::Object(map) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_wire_decoder() {
        // Construct sample GetPriceRequest wire format:
        // Tag 1 (partner_uid) = "eYrEAJNlz02c5Wz2Hf00"
        // Tag 2 (asset_type)  = "gold"
        let mut raw = vec![0x00, 0x00, 0x00, 0x00, 0x1E]; // gRPC 5-byte header
        // Tag 1 (key = 1 << 3 | 2 = 10 = 0x0A), len = 20 (0x14)
        raw.push(0x0A);
        raw.push(0x14);
        raw.extend_from_slice(b"eYrEAJNlz02c5Wz2Hf00");

        // Tag 2 (key = 2 << 3 | 2 = 18 = 0x12), len = 4 (0x04)
        raw.push(0x12);
        raw.push(0x04);
        raw.extend_from_slice(b"gold");

        let mut field_map = HashMap::new();
        field_map.insert(1, "partner_uid".to_string());
        field_map.insert(2, "asset_type".to_string());

        let res = decode_protobuf_wire(&raw[5..], Some(&field_map));
        assert_eq!(
            res,
            serde_json::json!({
                "partner_uid": "eYrEAJNlz02c5Wz2Hf00",
                "asset_type": "gold"
            })
        );
    }
}
