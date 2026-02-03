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
#[path = "test_conv.rs"]
mod tests;

use chrono::{DateTime, Utc};

/// Formats a DateTime<Utc> into a string with the given format.
pub fn time_parse(dt: DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}
