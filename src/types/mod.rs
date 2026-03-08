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
#[path = "test/arcx.rs"]
mod tests;

mod arcx;
pub use arcx::*;
pub use sqlx::types::*;
