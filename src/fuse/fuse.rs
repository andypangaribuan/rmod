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
pub use futures_util::future::BoxFuture;
use std::any::Any;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};

static MAX_BODY_SIZE: OnceLock<usize> = OnceLock::new();

pub type FuseResult = Result<(StatusCode, Arc<dyn Any + Send + Sync>), (StatusCode, Arc<dyn Any + Send + Sync>)>;
pub type FuseHandler = for<'a> fn(&'a mut FuseRContext) -> BoxFuture<'a, FuseResult>;

pub use rmod_macros::fuse_handler;

#[macro_export]
macro_rules! fuse_handlers {
    ([$($h:expr),* $(,)?]) => {
        vec![$($h as $crate::fuse::FuseHandler),*]
    };
    ($h:expr) => {
        vec![$h as $crate::fuse::FuseHandler]
    };
}

#[macro_export]
macro_rules! fuse_endpoints {
    ($($t:tt)*) => {
        {
            let mut map = ::std::collections::HashMap::<&'static str, Vec<$crate::fuse::FuseHandler>>::new();
            $crate::endpoints_inner!(map, $($t)*);
            map
        }
    };
}

#[macro_export]
macro_rules! endpoints_inner {
    ($map:ident $(,)?) => {};
    ($map:ident, $key:expr => [$($h:expr),* $(,)?], $($rest:tt)*) => {
        $map.insert($key, vec![$($h as $crate::fuse::FuseHandler),*]);
        $crate::endpoints_inner!($map, $($rest)*);
    };
    ($map:ident, $key:expr => $h:expr, $($rest:tt)*) => {
        $map.insert($key, vec![$h as $crate::fuse::FuseHandler]);
        $crate::endpoints_inner!($map, $($rest)*);
    };
    ($map:ident, $key:expr => [$($h:expr),* $(,)?]) => {
        $map.insert($key, vec![$($h as $crate::fuse::FuseHandler),*]);
    };
    ($map:ident, $key:expr => $h:expr) => {
        $map.insert($key, vec![$h as $crate::fuse::FuseHandler]);
    };
}

mod r_context_client_ip;

#[derive(Clone, Copy, Debug)]
pub struct FuseResSource {
    pub name: &'static str,
    pub handler_index: usize,
    pub endpoint_key: &'static str,
}

impl FuseResSource {
    pub(crate) fn new(name: &'static str) -> Self {
        Self { name, handler_index: 0, endpoint_key: "" }
    }
}

pub struct Fuse {
    router: Router,
}

pub struct FuseRContext {
    pub req: Request<Body>,
    pub data: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    pub res_status: Option<StatusCode>,
    pub res_body: Option<Arc<dyn Any + Send + Sync>>,
    pub res_backtrace: Option<Arc<Backtrace>>,
    pub res_source: FuseResSource,

    response: Option<Response>,
    pub body: Option<axum::body::Bytes>,
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

    pub fn endpoints(&mut self, precondition: Vec<FuseHandler>, defer: FuseHandler, mapping: HashMap<&'static str, Vec<FuseHandler>>) {
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

            let endpoint_key = key;
            let handlers = Arc::new(handlers);

            let precondition = Arc::new(precondition.clone());

            let limit = *MAX_BODY_SIZE.get_or_init(|| {
                std::env::var("RMOD_MAX_BODY_SIZE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100 * 1024 * 1024)
            });

            let handler_fn = move |req: Request<Body>| async move {
                let (parts, body) = req.into_parts();
                let bytes = axum::body::to_bytes(body, limit).await.unwrap_or_default();

                let mut ctx = FuseRContext::new(Request::from_parts(parts, Body::from(bytes.clone())));
                ctx.body = Some(bytes);

                ctx.res_handle(precondition, defer, handlers, endpoint_key).await
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

        let mut shutdown_rx = crate::util::lifecycle::subscribe();
        axum::serve(listener, self.router.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
            })
            .await
            .unwrap();

        crate::util::lifecycle::wait().await;
    }
}

impl FuseRContext {
    pub(crate) fn new(req: Request<Body>) -> Self {
        Self {
            req,
            data: Arc::new(Mutex::new(HashMap::new())),
            res_status: None,
            res_body: None,
            res_backtrace: None,
            res_source: FuseResSource::new(""),

            response: None,
            body: None,
        }
    }

    #[inline(never)]
    pub async fn res_handle(
        &mut self,
        precondition: Arc<Vec<FuseHandler>>,
        defer: FuseHandler,
        handlers: Arc<Vec<FuseHandler>>,
        endpoint_key: &'static str,
    ) -> Response {
        let mut break_next = false;

        for (i, h) in precondition.iter().enumerate() {
            match h(self).await {
                Ok((status, body)) => {
                    if !status.is_success() {
                        break_next = true;
                        self.res_status = Some(status);
                        self.res_body = Some(body);
                        self.res_source = FuseResSource { name: "precondition", handler_index: i, endpoint_key };
                    }
                }
                Err((status, body)) => {
                    break_next = true;
                    self.res_status = Some(status);
                    self.res_body = Some(body.clone());
                    if self.res_backtrace.is_none() {
                        self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
                    }
                    self.res_source = FuseResSource { name: "precondition", handler_index: i, endpoint_key };
                }
            }

            if break_next {
                break;
            }
        }

        if !break_next {
            for (i, h) in handlers.iter().enumerate() {
                match h(self).await {
                    Ok((status, body)) => {
                        self.res_status = Some(status);
                        self.res_body = Some(body);
                        self.res_source = FuseResSource { name: "handler", handler_index: i, endpoint_key };

                        if !status.is_success() {
                            break;
                        }
                    }
                    Err((status, body)) => {
                        self.res_status = Some(status);
                        self.res_body = Some(body.clone());
                        if self.res_backtrace.is_none() {
                            self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
                        }
                        self.res_source = FuseResSource { name: "handler", handler_index: i, endpoint_key };

                        break;
                    }
                }
            }
        }

        match defer(self).await {
            Ok((status, body)) => {
                self.res_status = Some(status);
                self.res_body = Some(body);
                self.res_source = FuseResSource { name: "defer", handler_index: 0, endpoint_key };
            }
            Err((status, body)) => {
                self.res_status = Some(status);
                self.res_body = Some(body.clone());
                if self.res_backtrace.is_none() {
                    self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
                }
                self.res_source = FuseResSource { name: "defer", handler_index: 0, endpoint_key };
            }
        }

        if let (Some(status), Some(body)) = (self.res_status, &self.res_body) {
            if let Some(text) = body.downcast_ref::<String>() {
                self.response = Some((status, text.clone()).into_response());
            } else if let Some(text) = body.downcast_ref::<&'static str>() {
                self.response = Some((status, (*text).to_string()).into_response());
            } else if let Some(json) = body.downcast_ref::<serde_json::Value>() {
                self.response = Some((status, axum::Json(json.clone())).into_response());
            }
        }

        self.response.take().unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
    }

    pub fn body_text(&self) -> String {
        if let Some(body) = &self.res_body {
            if let Some(s) = body.downcast_ref::<String>() {
                return s.clone();
            }
            if let Some(s) = body.downcast_ref::<&'static str>() {
                return (*s).to_string();
            }
            if let Some(e) = body.downcast_ref::<sqlx::Error>() {
                return e.to_string();
            }
        }
        "".to_string()
    }

    pub fn is_db_decode_error(&self) -> bool {
        if let Some(body) = &self.res_body
            && let Some(e) = body.downcast_ref::<sqlx::Error>()
        {
            return matches!(e, sqlx::Error::ColumnDecode { .. });
        }
        false
    }

    pub fn backtrace_text(&self) -> String {
        if let Some(bt) = &self.res_backtrace {
            return format!("{:?}", bt);
        }
        "".to_string()
    }

    #[inline(never)]
    pub fn backtrace_json(&self) -> serde_json::Value {
        if let Some(bt) = &self.res_backtrace {
            let bt_str = format!("{:#?}", bt);
            let mut frames = Vec::new();

            // 1. Try to parse the "raw" list format: { fn: "...", file: "...", line: ... }
            if bt_str.contains("{ fn: \"") {
                for frame_block in bt_str.split("{ fn: \"") {
                    let frame_block = frame_block.trim();
                    if frame_block.is_empty() || frame_block.starts_with("Backtrace") {
                        continue;
                    }

                    let mut frame_map = serde_json::Map::new();
                    if let Some(fn_end) = frame_block.find('\"') {
                        frame_map.insert("fn".to_string(), serde_json::json!(&frame_block[..fn_end]));

                        if let Some(file_start) = frame_block.find("file: \"") {
                            let file_str = &frame_block[file_start + 7..];
                            if let Some(file_end) = file_str.find('\"') {
                                let file_path = &file_str[..file_end];

                                if let Some(line_start) = file_str.find("line: ") {
                                    let line_str = &file_str[line_start + 6..];
                                    let line_end = line_str.find(|c: char| !c.is_numeric()).unwrap_or(line_str.len());
                                    let line_num = &line_str[..line_end];
                                    frame_map.insert("at".to_string(), serde_json::json!(format!("{}:{}", file_path, line_num)));
                                } else {
                                    frame_map.insert("at".to_string(), serde_json::json!(file_path));
                                }
                            }
                        }
                    }

                    if !frame_map.is_empty() {
                        frames.push(serde_json::Value::Object(frame_map));
                    }
                }
            }

            // 2. Fallback to the multi-line "at " format
            if frames.is_empty() {
                let mut current_frame: Option<serde_json::Map<String, serde_json::Value>> = None;
                for line in bt_str.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    if let Some(stripped) = line.strip_prefix("at ") {
                        if let Some(mut frame) = current_frame.take() {
                            frame.insert("at".to_string(), serde_json::json!(stripped.trim()));
                            frames.push(serde_json::Value::Object(frame));
                        }
                    } else {
                        if let Some(frame) = current_frame.take() {
                            frames.push(serde_json::Value::Object(frame));
                        }

                        let mut frame = serde_json::Map::new();
                        frame.insert("fn".to_string(), serde_json::json!(line));
                        current_frame = Some(frame);
                    }
                }
                if let Some(frame) = current_frame {
                    frames.push(serde_json::Value::Object(frame));
                }
            }

            let mut result = serde_json::json!(frames);
            if let Some(arr) = result.as_array_mut() {
                let mut first_rmod_idx = None;
                let mut last_relevant_idx = None;

                for (i, frame) in arr.iter().enumerate() {
                    if let Some(obj) = frame.as_object()
                        && let Some(f_name) = obj.get("fn").and_then(|v| v.as_str())
                        && f_name.contains("rmod::")
                    {
                        if first_rmod_idx.is_none() {
                            first_rmod_idx = Some(i);
                        }

                        last_relevant_idx = Some(i);
                    }
                }

                if let Some(start) = first_rmod_idx {
                    let end = last_relevant_idx.unwrap_or(start);
                    if start <= end {
                        *arr = arr[start..=end].to_vec();
                    } else {
                        *arr = arr[start..=start].to_vec();
                    }
                }
            }

            return result;
        }

        serde_json::json!([])
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

    #[inline(never)]
    pub fn ok<T: Send + Sync + 'static>(&mut self, status: StatusCode, body: T) -> FuseResult {
        self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
        Ok((status, Arc::new(body)))
    }

    #[inline(never)]
    pub fn err<T: Send + Sync + 'static>(&mut self, status: StatusCode, body: T) -> FuseResult {
        self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
        Err((status, Arc::new(body)))
    }

    #[inline(never)]
    pub fn ok_val<T: Send + Sync + 'static>(&mut self, status: StatusCode, body: T) -> (StatusCode, Arc<dyn Any + Send + Sync>) {
        self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
        (status, Arc::new(body))
    }

    #[inline(never)]
    pub fn err_val<T: Send + Sync + 'static>(&mut self, status: StatusCode, body: T) -> (StatusCode, Arc<dyn Any + Send + Sync>) {
        self.res_backtrace = Some(Arc::new(Backtrace::force_capture()));
        (status, Arc::new(body))
    }
}
