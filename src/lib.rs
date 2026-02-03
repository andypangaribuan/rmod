/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 * 
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub mod util;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
#[path = "test_lib.rs"]
mod tests;
