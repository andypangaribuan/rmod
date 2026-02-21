/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/http.rs"]
mod tests;

use once_cell::sync::Lazy;
use reqwest::{Client, Method, Response, header::HeaderMap};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

static CLIENTS: Lazy<Mutex<HashMap<String, Arc<Http>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

struct Http {
    client: Client,
}

impl Http {
    fn new() -> Self {
        Self::new_with_timeout(Duration::from_secs(30))
    }

    fn new_with_timeout(timeout: Duration) -> Self {
        let client = Client::builder().timeout(timeout).connect_timeout(Duration::from_secs(10)).build().unwrap_or_default();

        Self { client }
    }

    fn new_arc() -> Arc<Self> {
        Arc::new(Self::new())
    }

    fn new_arc_with_timeout(timeout: Duration) -> Arc<Self> {
        Arc::new(Self::new_with_timeout(timeout))
    }

    async fn get(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::GET, url, headers, query, None::<()>).await
    }

    async fn post<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::POST, url, headers, query, body).await
    }

    async fn put<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::PUT, url, headers, query, body).await
    }

    async fn patch<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::PATCH, url, headers, query, body).await
    }

    async fn delete(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::DELETE, url, headers, query, None::<()>).await
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        let mut rb = if let (Some(q), Ok(mut u)) = (query, reqwest::Url::parse(url)) {
            u.query_pairs_mut().extend_pairs(q.iter());
            self.client.request(method, u)
        } else {
            self.client.request(method, url)
        };

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
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

fn get_domain(url: &str) -> String {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_else(|| "default".to_string())
}

fn get_client(url: &str) -> Arc<Http> {
    let domain = get_domain(url);

    let mut clients = CLIENTS.lock().unwrap();
    clients.entry(domain).or_insert_with(Http::new_arc).clone()
}

pub fn client(url: &str, timeout: Duration) {
    let domain = get_domain(url);
    let mut clients = CLIENTS.lock().unwrap();
    clients.insert(domain, Http::new_arc_with_timeout(timeout));
}

pub async fn get(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
) -> Result<Response, reqwest::Error> {
    get_client(url).get(url, headers, query).await
}

pub async fn post<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    get_client(url).post(url, headers, query, body).await
}

pub async fn put<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    get_client(url).put(url, headers, query, body).await
}

pub async fn patch<T: Serialize>(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    get_client(url).patch(url, headers, query, body).await
}

pub async fn delete(
    url: &str,
    headers: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
) -> Result<Response, reqwest::Error> {
    get_client(url).delete(url, headers, query).await
}
