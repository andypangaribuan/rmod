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
#[should_panic(expected = "DATABASE_URL is not set")]
fn test_env_string_panic() {
    unsafe {
        env::remove_var("DATABASE_URL");
    }
    string("DATABASE_URL");
}
