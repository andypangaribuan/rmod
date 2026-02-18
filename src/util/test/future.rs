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
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_future_pool() {
    let mut pool = FuturePool::new();

    pool.add("task1", async {
        sleep(Duration::from_millis(100)).await;
        "result1"
    });

    pool.add("task2", async {
        sleep(Duration::from_millis(50)).await;
        "result2"
    });

    let results = pool.join_all().await;
    assert_eq!(results.len(), 2);

    // Check if both results are present (order might vary due to concurrency)
    let mut keys: Vec<&str> = results.iter().map(|(k, _)| *k).collect();
    keys.sort();
    assert_eq!(keys, vec!["task1", "task2"]);
}

#[tokio::test]
async fn test_future_pool_incremental() {
    let mut pool = FuturePool::new();

    pool.add(1, async {
        sleep(Duration::from_millis(30)).await;
        "a"
    });

    pool.add(2, async {
        sleep(Duration::from_millis(10)).await;
        "b"
    });

    // Task 2 should finish first
    let res1 = pool.join_next().await.unwrap();
    assert_eq!(res1, (2, "b"));

    let res2 = pool.join_next().await.unwrap();
    assert_eq!(res2, (1, "a"));

    assert!(pool.join_next().await.is_none());
}

#[tokio::test]
async fn test_future_pool_with_result() {
    let mut pool = FuturePool::new();
    pool.add("success", async { Ok::<i32, &str>(10) });
    pool.add("fail", async { Err::<i32, &str>("error message") });

    let results = pool.join_all().await;
    assert_eq!(results.len(), 2);

    for (key, res) in results {
        match key {
            "success" => assert_eq!(res, Ok(10)),
            "fail" => assert_eq!(res, Err("error message")),
            _ => panic!("unexpected key"),
        }
    }
}

#[tokio::test]
async fn test_future_pool_mixed_types() {
    #[derive(Debug, PartialEq)]
    enum Mixed {
        ValInt(i32),
        ValStr(&'static str),
    }

    let mut pool = FuturePool::new();
    pool.add("task_int", async { Mixed::ValInt(42) });
    pool.add("task_str", async { Mixed::ValStr("hello") });

    let results = pool.join_all().await;
    for (key, res) in results {
        match key {
            "task_int" => assert_eq!(res, Mixed::ValInt(42)),
            "task_str" => assert_eq!(res, Mixed::ValStr("hello")),
            _ => panic!("unexpected key"),
        }
    }
}

#[tokio::test]
async fn test_future_pool_with_struct() {
    #[derive(Debug, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    #[derive(Debug, PartialEq)]
    struct MyError {
        code: i32,
        message: String,
    }

    let mut pool = FuturePool::new();

    pool.add("user_1", async { Ok::<User, MyError>(User { id: 1, name: "Andy".into() }) });

    pool.add("user_2", async { Err::<User, MyError>(MyError { code: 404, message: "Not Found".into() }) });

    let results = pool.join_all().await;
    assert_eq!(results.len(), 2);

    for (key, res) in results {
        match key {
            "user_1" => {
                assert_eq!(res, Ok(User { id: 1, name: "Andy".into() }));
            }
            "user_2" => {
                assert_eq!(res, Err(MyError { code: 404, message: "Not Found".into() }));
            }
            _ => panic!("unexpected key"),
        }
    }
}

#[tokio::test]
async fn test_future_pool_pass_values() {
    let mut pool = FuturePool::new();

    let input_value = "input data".to_string();
    let multiplier = 2;

    pool.add("task_1", async move { format!("{}-{}", input_value, multiplier) });

    let results = pool.join_all().await;
    assert_eq!(results[0].1, "input data-2");
}

#[tokio::test]
async fn test_future_pool_vector_loop() {
    let mut pool = FuturePool::new();
    let fruits = vec!["apple", "banana", "orange"];

    for (idx, fruit) in fruits.into_iter().enumerate() {
        pool.add(idx, async move { format!("{}-{}", idx, fruit) });
    }

    let results = pool.join_all().await;
    assert_eq!(results.len(), 3);

    for (idx, res) in results {
        match idx {
            0 => assert_eq!(res, "0-apple"),
            1 => assert_eq!(res, "1-banana"),
            2 => assert_eq!(res, "2-orange"),
            _ => panic!("unexpected index"),
        }
    }
}

#[tokio::test]
async fn test_future_burst() {
    use crate::util::FutureBurst;

    let data = vec!["a", "b", "c", "d", "e"];
    let max_parallel = 2;

    let results = FutureBurst::run(data, max_parallel, |idx, val| async move {
        println!("start : idx={}, val={}", idx, val);
        let ms = rand::random_range(50..=100);
        sleep(Duration::from_millis(ms)).await;
        let new_val = val.to_uppercase();
        println!("end   : idx={}, val={}", idx, new_val);
        new_val
    })
    .await;

    assert_eq!(results.len(), 5);

    let mut values: Vec<String> = results.into_iter().map(|(_, v)| v).collect();
    values.sort();
    assert_eq!(values, vec!["A", "B", "C", "D", "E"]);
}
