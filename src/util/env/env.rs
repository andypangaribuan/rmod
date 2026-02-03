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
#[path = "test_env.rs"]
mod tests;

use std::env;
use std::fmt::Display;
use std::str::FromStr;

/// Gets an environment variable as a String.
/// Panics with a clear message if not set.
pub fn string(name: &str) -> String {
    match env::var(name) {
        Ok(v) => v,
        Err(_) => panic!("{} is not set", name),
    }
}

/// Gets an environment variable as a String, or returns a default value if not set.
pub fn string_or(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(v) => v,
        Err(_) => default.to_string(),
    }
}

/// Gets an environment variable as a generic type T (e.g., u16, u32).
/// Panics if not set or if parsing fails.
pub fn int<T>(name: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let val = string(name);
    val.parse::<T>()
        .unwrap_or_else(|e| panic!("failed to parse {} as integer, value: {}, error: {}", name, val, e))
}

/// Gets an environment variable as a generic type T, or returns a default value if not set or parsing fails.
pub fn int_or<T>(name: &str, default: T) -> T
where
    T: FromStr,
{
    match env::var(name) {
        Ok(v) => v.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}
