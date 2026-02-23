/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub(crate) fn build_select_sql<T>(table_name: &str, where_clause: &str, opt: Option<&crate::db::Opt<T>>) -> String {
    let table_name = opt.and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table_name);
    let with_deleted_at = opt.and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);
    let mut sql = if where_clause.trim().is_empty() {
        if with_deleted_at {
            format!("SELECT * FROM {} WHERE deleted_at IS NULL", table_name)
        } else {
            format!("SELECT * FROM {}", table_name)
        }
    } else if with_deleted_at {
        format!("SELECT * FROM {} WHERE ({}) AND deleted_at IS NULL", table_name, where_clause)
    } else {
        format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
    };

    if let Some(tail_query) = opt.and_then(|o| o.tail_query.as_ref()) {
        sql.push(' ');
        sql.push_str(tail_query);
    }

    sql
}
