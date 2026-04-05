/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use axum::http::StatusCode;
use dashmap::DashMap;
use reqwest::{Client, Method, Response, header::HeaderMap};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

use std::sync::LazyLock;

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
        HTTP_CLIENT.request(method, u)
    } else {
        HTTP_CLIENT.request(method, url)
    };

    rb = rb.timeout(timeout);

    if let Some(h) = headers {
        let mut head_map = HeaderMap::new();
        for (k, v) in h {
            if let (Ok(ref name), Ok(ref value)) =
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()), reqwest::header::HeaderValue::from_str(&v))
            {
                head_map.insert(name.clone(), value.clone());
            }
        }
        rb = rb.headers(head_map);
    }

    if let Some(b) = body {
        rb = rb.json(&b);
    }

    rb.send().await
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
