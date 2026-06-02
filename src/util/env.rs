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
#[path = "test/env.rs"]
mod tests;

use crate::fct::FCT;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::str::FromStr;

fn parse_zx_env() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(zx_env) = env::var("ZX_ENV") {
        for line in zx_env.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let val = val.trim();

                let mut parsed_val = val;
                if let Some(stripped) = val.strip_prefix('"') {
                    if let Some(end_idx) = stripped.find('"') {
                        parsed_val = &stripped[..end_idx];
                    } else {
                        let mut temp = stripped;
                        if let Some(comment_idx) = temp.find('#') {
                            temp = &temp[..comment_idx];
                        }
                        parsed_val = temp;
                    }
                } else if let Some(stripped) = val.strip_prefix('\'') {
                    if let Some(end_idx) = stripped.find('\'') {
                        parsed_val = &stripped[..end_idx];
                    } else {
                        let mut temp = stripped;
                        if let Some(comment_idx) = temp.find('#') {
                            temp = &temp[..comment_idx];
                        }
                        parsed_val = temp;
                    }
                } else if let Some(comment_idx) = val.find('#') {
                    parsed_val = &val[..comment_idx];
                }

                map.insert(key, parsed_val.trim().to_string());
            }
        }
    }
    map
}

fn var(name: &str) -> Result<String, env::VarError> {
    match env::var(name) {
        Ok(v) => Ok(v),
        Err(e) => {
            if name == "ZX_ENV" {
                return Err(e);
            }
            match parse_zx_env().get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(e),
            }
        }
    }
}

/// Gets an environment variable as a String.
/// Panics with a clear message if not set.
pub fn string(name: &str) -> String {
    match var(name) {
        Ok(v) => v,
        Err(_) => panic!("{} is not set", name),
    }
}

/// Gets an environment variable as a String, or returns a default value if not set.
pub fn string_or(name: &str, default: &str) -> String {
    match var(name) {
        Ok(v) => v,
        Err(_) => default.to_string(),
    }
}

/// Gets an environment variable as an Option<String>.
pub fn string_opt(name: &str) -> Option<String> {
    var(name).ok()
}

pub trait EnvParsable: FromStr
where
    <Self as FromStr>::Err: Display,
{
}
impl EnvParsable for i16 {}
impl EnvParsable for i32 {}
impl EnvParsable for u16 {}
impl EnvParsable for u32 {}
impl EnvParsable for String {}
impl EnvParsable for FCT {}
impl EnvParsable for bool {}

/// Gets an environment variable as a generic type T.
/// Panics if not set or if parsing fails.
pub fn int<T>(name: &str) -> T
where
    T: EnvParsable,
    <T as FromStr>::Err: Display,
{
    let val = string(name);
    val.parse::<T>().unwrap_or_else(|e| panic!("failed to parse {} as requested type, value: {}, error: {}", name, val, e))
}

/// Gets an environment variable as a generic type T, or returns a default value if not set or parsing fails.
pub fn int_or<T>(name: &str, default: T) -> T
where
    T: EnvParsable,
    <T as FromStr>::Err: Display,
{
    match var(name) {
        Ok(v) => v.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

/// Gets an environment variable as a generic type T, or returns None if not set or parsing fails.
pub fn int_opt<T>(name: &str) -> Option<T>
where
    T: EnvParsable,
    <T as FromStr>::Err: Display,
{
    var(name).ok().and_then(|v| v.parse::<T>().ok())
}

/// Gets an environment variable as FCT.
/// Panics if not set or if parsing fails.
pub fn fct(name: &str) -> FCT {
    let val = string(name);
    FCT(Decimal::from_str(&val).unwrap_or_else(|e| panic!("failed to parse {} as fct, value: {}, error: {}", name, val, e)))
}

/// Gets an environment variable as FCT, or returns a default value if not set or parsing fails.
pub fn fct_or(name: &str, default: FCT) -> FCT {
    match var(name) {
        Ok(v) => FCT(Decimal::from_str(&v).unwrap_or(*default)),
        Err(_) => default,
    }
}

/// Gets an environment variable as a Vec<T>, splitting by the given separator.
/// Panics if not set or if parsing any part fails.
pub fn ls<T>(name: &str, sep: &str) -> Vec<T>
where
    T: EnvParsable,
    <T as FromStr>::Err: Display,
{
    let val = string(name);

    if val.is_empty() {
        return vec![];
    }

    val.split(sep)
        .map(|s| {
            s.parse::<T>().unwrap_or_else(|e| panic!("failed to parse part of {} as requested type, value: {}, error: {}", name, s, e))
        })
        .collect()
}

/// Gets an environment variable as a bool.
/// Panics if not set or if parsing fails.
pub fn bool(name: &str) -> bool {
    let val = string(name);
    val.parse::<bool>().unwrap_or_else(|e| panic!("failed to parse {} as bool, value: {}, error: {}", name, val, e))
}

/// Gets an environment variable as a bool, or returns a default value if not set or parsing fails.
pub fn bool_or(name: &str, default: bool) -> bool {
    match var(name) {
        Ok(v) => v.parse::<bool>().unwrap_or(default),
        Err(_) => default,
    }
}

/// Gets an environment variable as an Option<bool>.
pub fn bool_opt(name: &str) -> Option<bool> {
    var(name).ok().and_then(|v| v.parse::<bool>().ok())
}
