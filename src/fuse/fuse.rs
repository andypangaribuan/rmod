/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use axum;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{MethodFilter, Router, on},
};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type FuseResult = Result<(StatusCode, Arc<dyn Any + Send + Sync>), (StatusCode, Arc<dyn Any + Send + Sync>)>;
pub type FuseHandler = fn(&mut FuseRContext) -> FuseResult;

pub struct Fuse {
    router: Router,
}

pub struct FuseRContext {
    pub req: Request<Body>,
    pub data: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    pub res_status: Option<StatusCode>,
    pub res_body: Option<Arc<dyn Any + Send + Sync>>,
    res_source: String,
    response: Option<Response>,
    pub body: Option<Vec<u8>>,
}

impl Default for Fuse {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn rest<F, S>(addr: &str, f: F, on_start: Option<S>)
where
    F: FnOnce(&mut Fuse),
    S: FnOnce(),
{
    let mut fuse = Fuse::new();
    f(&mut fuse);
    fuse.run(addr, on_start).await;
}

impl Fuse {
    pub(crate) fn new() -> Self {
        Self { router: Router::new() }
    }

    pub fn endpoints(
        &mut self,
        liveness: FuseHandler,
        authentication: Option<FuseHandler>,
        defer: FuseHandler,
        mapping: HashMap<&str, Vec<FuseHandler>>,
    ) {
        for (key, handlers) in mapping {
            let parts: Vec<&str> = key.split(": ").collect();
            if parts.len() != 2 {
                continue;
            }

            let method_str = parts[0];
            let path = parts[1].to_string();

            let filter = match method_str {
                "GET" => MethodFilter::GET,
                "POS" | "POST" => MethodFilter::POST,
                "PUT" => MethodFilter::PUT,
                "DEL" | "DELETE" => MethodFilter::DELETE,
                "PAT" | "PATCH" => MethodFilter::PATCH,
                _ => MethodFilter::GET,
            };

            let endpoint_key = key.to_string();
            let handlers = Arc::new(handlers);

            let handler_fn = move |req: Request<Body>| async move {
                let (parts, body) = req.into_parts();
                let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default().to_vec();

                let mut ctx = FuseRContext::new(Request::from_parts(parts, Body::from(bytes.clone())));
                ctx.body = Some(bytes);

                let mut break_next = false;

                // 1. Liveness
                match liveness(&mut ctx) {
                    Ok((status, body)) | Err((status, body)) => {
                        if !status.is_success() {
                            break_next = true;
                            ctx.res_status = Some(status);
                            ctx.res_body = Some(body);
                            ctx.res_source = "liveness".to_string();
                        }
                    }
                }

                // 2. Authentication
                if !break_next && let Some(v) = authentication {
                    match v(&mut ctx) {
                        Ok((status, body)) | Err((status, body)) => {
                            if !status.is_success() {
                                break_next = true;
                                ctx.res_status = Some(status);
                                ctx.res_body = Some(body);
                                ctx.res_source = "authentication".to_string();
                            }
                        }
                    }
                }

                // 3. Handlers
                if !break_next {
                    for (i, h) in handlers.iter().enumerate() {
                        match h(&mut ctx) {
                            Ok((status, body)) | Err((status, body)) => {
                                ctx.res_status = Some(status);
                                ctx.res_body = Some(body);
                                ctx.res_source = format!("handler:{}:{}", i, endpoint_key);

                                if !status.is_success() {
                                    break;
                                }
                            }
                        }
                    }
                }

                // 3. Defer
                match defer(&mut ctx) {
                    Ok((status, body)) | Err((status, body)) => {
                        ctx.res_status = Some(status);
                        ctx.res_body = Some(body);
                        ctx.res_source = "defer".to_string();
                    }
                }

                if let (Some(status), Some(body)) = (ctx.res_status, &ctx.res_body) {
                    if let Some(text) = body.downcast_ref::<String>() {
                        ctx.response = Some((status, text.clone()).into_response());
                    } else if let Some(text) = body.downcast_ref::<&'static str>() {
                        ctx.response = Some((status, (*text).to_string()).into_response());
                    }
                }

                ctx.response.unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
            };

            let router = std::mem::take(&mut self.router);
            self.router = router.route(&path, on(filter, handler_fn));
        }
    }

    pub(crate) async fn run<F: FnOnce()>(self, addr: &str, on_start: Option<F>) {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        if let Some(f) = on_start {
            f();
        }
        axum::serve(listener, self.router).await.unwrap();
    }
}

impl FuseRContext {
    pub(crate) fn new(req: Request<Body>) -> Self {
        Self {
            req,
            data: Arc::new(Mutex::new(HashMap::new())),
            res_status: None,
            res_body: None,
            res_source: "".to_string(),
            response: None,
            body: None,
        }
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_path_to_error::Error<serde_json::Error>> {
        let bytes = self.body.as_deref().unwrap_or(&[]);
        let mut de = serde_json::Deserializer::from_slice(bytes);
        serde_path_to_error::deserialize(&mut de)
    }

    pub fn set<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), Arc::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<Arc<T>> {
        let data = self.data.lock().unwrap();
        data.get(key)?.clone().downcast::<T>().ok()
    }

    pub fn ok<T: Send + Sync + 'static>(&self, status: StatusCode, body: T) -> FuseResult {
        Ok((status, Arc::new(body)))
    }

    pub fn err<T: Send + Sync + 'static>(&self, status: StatusCode, body: T) -> FuseResult {
        Err((status, Arc::new(body)))
    }

    pub fn res_source(&self) -> &str {
        &self.res_source
    }
}
