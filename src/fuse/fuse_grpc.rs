/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

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

            let endpoint_uid = crate::uid::new();

            let clog_config = crate::clog::get_config();
            let service_name = clog_config.map(|c| c.service_name.clone()).unwrap_or_default();

            let is_excluded =
                is_health_check || clog_config.map(|c| c.exclusion_routes.iter().any(|r| path.starts_with(r))).unwrap_or(false);

            let log_ctx = crate::clog::LogContext {
                trace_id: trace_id.clone(),
                parent_uid: parent_uid.clone(),
                endpoint_uid: endpoint_uid.clone(),
                service_name: service_name.clone(),
            };

            let start_time = std::time::Instant::now();
            let limit = std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024);

            let req_axum_body = axum::body::Body::new(body);
            let req_bytes = axum::body::to_bytes(req_axum_body, limit).await.unwrap_or_default();
            let req_body_box =
                tonic::body::BoxBody::new(http_body_util::Full::new(req_bytes.clone()).map_err(|_| tonic::Status::internal("body error")));
            let req_reconstructed = tonic::codegen::http::Request::from_parts(parts, req_body_box);

            let res_result = crate::clog::LOG_CTX.scope(log_ctx, inner.call(req_reconstructed)).await;

            match res_result {
                Ok(response) => {
                    let (res_parts, res_body) = response.into_parts();
                    let duration_ms = start_time.elapsed().as_millis() as i32;

                    let grpc_status = res_parts.headers.get("grpc-status").and_then(|v| v.to_str().ok()).unwrap_or("0").to_string();

                    let status_code = if grpc_status == "0" || res_parts.status.is_success() { 200 } else { 500 };

                    let res_axum_body = axum::body::Body::new(res_body);
                    let res_bytes = axum::body::to_bytes(res_axum_body, limit).await.unwrap_or_default();

                    if !is_excluded && clog_config.is_some() {
                        let format_payload = |bytes: &bytes::Bytes| -> String {
                            if bytes.is_empty() {
                                String::new()
                            } else {
                                let payload = if bytes.len() > 5 { &bytes[5..] } else { &bytes[..] };
                                let s = String::from_utf8_lossy(payload);
                                if s.len() > 100_000 { format!("{}... [TRUNCATED]", &s[..100_000]) } else { s.to_string() }
                            }
                        };

                        let req_body_truncated = format_payload(&req_bytes);
                        let res_body_truncated = format_payload(&res_bytes);

                        let payload_json = serde_json::json!({
                            "endpoint": path,
                            "path": path,
                            "request_body": req_body_truncated,
                            "response_body": res_body_truncated,
                            "grpc_status": grpc_status,
                        })
                        .to_string();

                        let now_ms = crate::time::now_ms();
                        crate::clog::push_log(crate::clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_ms: now_ms,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            log_type: "GRPC_INCOMING".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code,
                            payload_json,
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
            let ctx = crate::clog::get_current_ctx();
            let trace_id = match ctx {
                Some(ref c) => c.trace_id.clone(),
                None => crate::uid::new(),
            };
            let parent_uid = ctx.as_ref().map(|c| c.endpoint_uid.clone());
            let endpoint_uid = crate::uid::new();
            let service_name = ctx.as_ref().map(|c| c.service_name.clone()).unwrap_or_default();

            if let Ok(v) = tonic::codegen::http::HeaderValue::from_str(&trace_id) {
                req.headers_mut().insert("x-trace-id", v);
            }
            if let Some(ref p_uid) = parent_uid
                && let Ok(v) = tonic::codegen::http::HeaderValue::from_str(p_uid)
            {
                req.headers_mut().insert("x-parent-uid", v);
            }

            let (parts, body) = req.into_parts();
            let path = parts.uri.path().to_string();

            let is_health_check = path.starts_with("/grpc.health.v1.Health");
            let clog_config = crate::clog::get_config();
            let is_excluded =
                is_health_check || clog_config.map(|c| c.exclusion_routes.iter().any(|r| path.starts_with(r))).unwrap_or(false);

            let start_time = std::time::Instant::now();
            let limit = std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024);

            let req_axum_body = axum::body::Body::new(body);
            let req_bytes = axum::body::to_bytes(req_axum_body, limit).await.unwrap_or_default();
            let req_body_box =
                tonic::body::BoxBody::new(http_body_util::Full::new(req_bytes.clone()).map_err(|_| tonic::Status::internal("body error")));
            let req_reconstructed = tonic::codegen::http::Request::from_parts(parts, req_body_box);

            let res_result = inner.call(req_reconstructed).await;

            match res_result {
                Ok(response) => {
                    let (res_parts, res_body) = response.into_parts();
                    let duration_ms = start_time.elapsed().as_millis() as i32;

                    let grpc_status = res_parts.headers.get("grpc-status").and_then(|v| v.to_str().ok()).unwrap_or("0").to_string();

                    let status_code = if grpc_status == "0" || res_parts.status.is_success() { 200 } else { 500 };

                    let res_axum_body = axum::body::Body::new(res_body);
                    let res_bytes = axum::body::to_bytes(res_axum_body, limit).await.unwrap_or_default();

                    if !is_excluded && clog_config.is_some() {
                        let format_payload = |bytes: &bytes::Bytes| -> String {
                            if bytes.is_empty() {
                                String::new()
                            } else {
                                let payload = if bytes.len() > 5 { &bytes[5..] } else { &bytes[..] };
                                let s = String::from_utf8_lossy(payload);
                                if s.len() > 100_000 { format!("{}... [TRUNCATED]", &s[..100_000]) } else { s.to_string() }
                            }
                        };

                        let req_body_truncated = format_payload(&req_bytes);
                        let res_body_truncated = format_payload(&res_bytes);

                        let mut payload_map = serde_json::json!({
                            "endpoint": path,
                            "path": path,
                            "request_body": req_body_truncated,
                            "response_body": res_body_truncated,
                            "grpc_status": grpc_status,
                        });

                        if status_code != 200 {
                            let bt = std::backtrace::Backtrace::force_capture();
                            let bt_str = format!("{}", bt);
                            if !bt_str.trim().is_empty() {
                                payload_map["stacktrace"] = serde_json::Value::String(bt_str);
                            }
                        }

                        let payload_json = payload_map.to_string();

                        let now_ms = crate::time::now_ms();
                        crate::clog::push_log(crate::clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_ms: now_ms,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            log_type: "GRPC_OUTGOING".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code,
                            payload_json,
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
                        if !bt_str.trim().is_empty() {
                            payload_map["stacktrace"] = serde_json::Value::String(bt_str);
                        }

                        let now_ms = crate::time::now_ms();
                        crate::clog::push_log(crate::clog::LogEntry {
                            uid: endpoint_uid,
                            timestamp_unix_ms: now_ms,
                            service_name,
                            trace_id,
                            parent_uid: parent_uid.unwrap_or_default(),
                            log_type: "GRPC_OUTGOING".to_string(),
                            action_name: path.clone(),
                            duration_ms,
                            status_code: 500,
                            payload_json: payload_map.to_string(),
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
