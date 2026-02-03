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

/// Gets an environment variable as a String.
/// Panics with a clear message if not set and no default value is provided.
pub fn string(name: &str, default_value: Option<&str>) -> String {
    match env::var(name) {
        Ok(v) => v,
        Err(_) => match default_value {
            Some(v) => v.to_string(),
            None => panic!("{} is not set", name),
        },
    }
}
