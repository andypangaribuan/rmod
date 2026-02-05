/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::*;
use std::env;

#[test]
fn test_env_string() {
    unsafe {
        env::set_var("APP_NAME", "my-awesome-app");
    }
    assert_eq!(string("APP_NAME"), "my-awesome-app");
}

#[test]
fn test_env_string_default() {
    unsafe {
        env::remove_var("APP_VERSION");
    }
    assert_eq!(string_or("APP_VERSION", "1.0.0"), "1.0.0");
}

#[test]
#[should_panic(expected = "DATABASE_URL is not set")]
fn test_env_string_panic() {
    unsafe {
        env::remove_var("DATABASE_URL");
    }
    string("DATABASE_URL");
}

#[test]
fn test_env_int() {
    unsafe {
        env::set_var("APP_PORT", "8080");
        env::set_var("TIMEOUT", "30");
    }
    let port: i16 = int("APP_PORT");
    let timeout: i32 = int("TIMEOUT");
    assert_eq!(port, 8080);
    assert_eq!(timeout, 30);
}

#[test]
fn test_env_int_default() {
    unsafe {
        env::remove_var("RETRY_COUNT");
        env::set_var("MAX_CONNECTIONS", "invalid");
    }
    // Missing variable
    let retry: i32 = int_or("RETRY_COUNT", 5);
    // Invalid variable value
    let max_conn: i16 = int_or("MAX_CONNECTIONS", 100);

    assert_eq!(retry, 5);
    assert_eq!(max_conn, 100);
}

#[test]
#[should_panic(expected = "failed to parse INVALID_INT as integer")]
fn test_env_int_panic() {
    unsafe {
        env::set_var("INVALID_INT", "not-a-number");
    }
    let _: i32 = int("INVALID_INT");
}
