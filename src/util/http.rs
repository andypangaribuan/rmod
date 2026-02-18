/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use reqwest::{Client, Method, Response, header::HeaderMap};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
#[path = "test/http.rs"]
mod tests;

pub struct Http {
    client: Client,
}

impl Http {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn builder() -> reqwest::ClientBuilder {
        Client::builder()
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn get(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<Response, reqwest::Error> {
        self.request(Method::GET, url, headers, None::<()>).await
    }

    pub async fn post<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::POST, url, headers, body).await
    }

    pub async fn put<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::PUT, url, headers, body).await
    }

    pub async fn patch<T: Serialize>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        self.request(Method::PATCH, url, headers, body).await
    }

    pub async fn delete(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<Response, reqwest::Error> {
        self.request(Method::DELETE, url, headers, None::<()>).await
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<T>,
    ) -> Result<Response, reqwest::Error> {
        let mut rb = self.client.request(method, url);

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
