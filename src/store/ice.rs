/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub fn is_db_exists(key: &str) -> bool {
    super::db_exists(key)
}

pub fn get_db_state(key: &str) -> String {
    super::db_state(key)
}

pub fn set_db_with_deleted_at(val: bool) {
    super::update_db_with_deleted_at(val)
}
