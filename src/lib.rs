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
#[path = "test_lib.rs"]
mod tests;

#[path = "mod_util.rs"]
pub mod util;

#[path = "config/config.rs"]
pub mod config;

#[path = "store/store.rs"]
pub mod store;

#[path = "fuse/fuse.rs"]
pub mod fuse;

#[path = "db/db.rs"]
pub mod db;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
