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

/// Gets an environment variable as an Option<String>.
pub fn string_opt(name: &str) -> Option<String> {
    env::var(name).ok()
}

pub trait Integer: FromStr
where
    <Self as FromStr>::Err: Display,
{
}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for u16 {}
impl Integer for u32 {}

/// Gets an environment variable as a generic type T (only u16 or u32).
/// Panics if not set or if parsing fails.
pub fn int<T>(name: &str) -> T
where
    T: Integer,
    <T as FromStr>::Err: Display,
{
    let val = string(name);
    val.parse::<T>().unwrap_or_else(|e| panic!("failed to parse {} as integer, value: {}, error: {}", name, val, e))
}

/// Gets an environment variable as a generic type T (only u16 or u32), or returns a default value if not set or parsing fails.
pub fn int_or<T>(name: &str, default: T) -> T
where
    T: Integer,
    <T as FromStr>::Err: Display,
{
    match env::var(name) {
        Ok(v) => v.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

/// Gets an environment variable as a generic type T, or returns None if not set or parsing fails.
pub fn int_opt<T>(name: &str) -> Option<T>
where
    T: Integer,
    <T as FromStr>::Err: Display,
{
    env::var(name).ok().and_then(|v| v.parse::<T>().ok())
}
