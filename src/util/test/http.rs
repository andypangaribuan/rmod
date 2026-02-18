/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::util::FuturePool;
use crate::util::future_burst;
use crate::util::http;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_http_get() {
    let url = "https://httpbin.org/get";

    let mut headers = HashMap::new();
    headers.insert("X-Test-Header".to_string(), "test-value".to_string());

    let res = http::get(url, Some(headers)).await.unwrap();
    assert!(res.status().is_success());

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["headers"]["X-Test-Header"], "test-value");
}

#[tokio::test]
async fn test_http_post() {
    let url = "https://httpbin.org/post";

    let body = serde_json::json!({
        "name": "Andy",
        "role": "Developer"
    });

    let res = http::post(url, None, Some(body)).await.unwrap();
    assert!(res.status().is_success());

    let res_body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(res_body["json"]["name"], "Andy");
}

#[tokio::test]
async fn test_http_parallel_requests() {
    let mut pool = FuturePool::new();
    let urls = vec!["https://httpbin.org/get", "https://httpbin.org/ip", "https://httpbin.org/user-agent"];

    for url in urls {
        pool.add(url, async move { http::get(url, None).await });
    }

    let results = pool.join_all().await;
    assert_eq!(results.len(), 3);

    for (url, res) in results {
        let response = res.expect("Request failed");
        assert!(response.status().is_success(), "Failed for URL: {}", url);
    }
}

#[tokio::test]
async fn test_http_future_burst() {
    let urls = vec!["https://httpbin.org/get", "https://httpbin.org/ip", "https://httpbin.org/user-agent", "https://httpbin.org/headers"];

    let results = future_burst(urls, 2, |idx, url| async move {
        println!("idx: {}, calling: {}", idx, url);
        let res = http::get(url, None).await;
        println!("idx: {}, done   : {}", idx, url);
        res
    })
    .await;

    assert_eq!(results.len(), 4);

    for (idx, res) in results {
        let response = res.expect("Request failed");
        assert!(response.status().is_success(), "Failed at index: {}", idx);
    }
}

#[tokio::test]
async fn test_http_smart_functions() {
    let url = "https://httpbin.org/get";
    let res = http::get(url, None).await.unwrap();
    assert!(res.status().is_success());

    let url_post = "https://httpbin.org/post";
    let body = serde_json::json!({ "smart": true });
    let res_post = http::post(url_post, None, Some(body)).await.unwrap();
    assert!(res_post.status().is_success());
}

#[tokio::test]
async fn test_http_custom_timeout() {
    let url = "https://timeout.httpbin.org/get";
    // Register a client for this domain with a very short timeout
    http::client(url, Duration::from_millis(1));

    // This should now fail with a timeout error
    let res = http::get(url, None).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().is_timeout());
}
