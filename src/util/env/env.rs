/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 * 
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::env;

/// Gets an environment variable as a String.
/// Panics with a clear message if not set.
pub fn string(name: &str) -> String {
    env::var(name).expect(&format!("{} is not set", name))
}

#[cfg(test)]
#[path = "test_env.rs"]
mod tests;
