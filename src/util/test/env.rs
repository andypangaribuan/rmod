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
#[should_panic(expected = "failed to parse INVALID_INT as requested type")]
fn test_env_int_panic() {
    unsafe {
        env::set_var("INVALID_INT", "not-a-number");
    }
    let _: i32 = int("INVALID_INT");
}

#[test]
fn test_env_fct() {
    unsafe {
        env::set_var("APP_PRICE", "123.45");
    }
    let price = fct("APP_PRICE");
    assert_eq!(*price, Decimal::from_str("123.45").unwrap());
}

#[test]
fn test_env_fct_default() {
    unsafe {
        env::remove_var("DEFAULT_PRICE");
        env::set_var("INVALID_PRICE", "not-a-decimal");
    }
    let default_val = FCT(Decimal::new(10, 1)); // 1.0

    let price1 = fct_or("DEFAULT_PRICE", default_val);
    let price2 = fct_or("INVALID_PRICE", default_val);

    assert_eq!(price1, default_val);
    assert_eq!(price2, default_val);
}

#[test]
fn test_env_ls_string() {
    unsafe {
        env::set_var("APP_MODULES", "auth,user,order");
    }
    let modules: Vec<String> = ls("APP_MODULES", ",");
    assert_eq!(modules, vec!["auth".to_string(), "user".to_string(), "order".to_string()]);
}

#[test]
fn test_env_ls_u16() {
    unsafe {
        env::set_var("APP_PORTS", "8080, 8081, 8082");
    }
    let ports: Vec<u16> = ls("APP_PORTS", ", ");
    assert_eq!(ports, vec![8080, 8081, 8082]);
}

#[test]
fn test_env_ls_fct() {
    unsafe {
        env::set_var("APP_PRICES", "10.0|20.5|30.9");
    }
    let prices: Vec<FCT> = ls("APP_PRICES", "|");
    assert_eq!(prices.len(), 3);
    assert_eq!(*prices[0], Decimal::from_str("10.0").unwrap());
    assert_eq!(*prices[1], Decimal::from_str("20.5").unwrap());
    assert_eq!(*prices[2], Decimal::from_str("30.9").unwrap());
}

#[test]
fn test_env_bool() {
    unsafe {
        env::set_var("DEBUG_MODE", "true");
        env::set_var("FEATURE_ENABLED", "false");
    }
    assert!(bool("DEBUG_MODE"));
    assert!(!bool("FEATURE_ENABLED"));
    assert_eq!(bool_opt("DEBUG_MODE"), Some(true));
    assert_eq!(bool_opt("FEATURE_ENABLED"), Some(false));
    assert_eq!(bool_opt("NON_EXISTENT"), None);
}

#[test]
fn test_env_bool_default() {
    unsafe {
        env::remove_var("MISSING_BOOL");
        env::set_var("INVALID_BOOL", "not-a-bool");
    }
    assert!(bool_or("MISSING_BOOL", true));
    assert!(!bool_or("MISSING_BOOL", false));
    assert!(bool_or("INVALID_BOOL", true));
    assert!(!bool_or("INVALID_BOOL", false));
}

#[test]
fn test_env_ls_bool() {
    unsafe {
        env::set_var("BOOL_LIST", "true,false,true");
    }
    let list: Vec<bool> = ls("BOOL_LIST", ",");
    assert_eq!(list, vec![true, false, true]);
}
