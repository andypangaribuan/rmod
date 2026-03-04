/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::{OnceLock, RwLock};

static TIMEZONE: OnceLock<RwLock<Option<String>>> = OnceLock::new();

pub(crate) fn update_timezone(val: String) {
    let lock = TIMEZONE.get_or_init(|| RwLock::new(None));
    let mut store = lock.write().unwrap();
    *store = Some(val);
}

pub(crate) fn get_timezone() -> Option<String> {
    TIMEZONE.get()?.read().ok()?.clone()
}
