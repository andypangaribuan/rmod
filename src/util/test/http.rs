/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::util::Http;
use std::collections::HashMap;

#[tokio::test]
async fn test_http_get() {
    let http = Http::new();
    // Using a reliable public API for testing
    let url = "https://httpbin.org/get";

    let mut headers = HashMap::new();
    headers.insert("X-Test-Header".to_string(), "test-value".to_string());

    let res = http.get(url, Some(headers)).await.unwrap();
    assert!(res.status().is_success());

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["headers"]["X-Test-Header"], "test-value");
}

#[tokio::test]
async fn test_http_post() {
    let http = Http::new();
    let url = "https://httpbin.org/post";

    let body = serde_json::json!({
        "name": "Andy",
        "role": "Developer"
    });

    let res = http.post(url, None, Some(body)).await.unwrap();
    assert!(res.status().is_success());

    let res_body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(res_body["json"]["name"], "Andy");
}
