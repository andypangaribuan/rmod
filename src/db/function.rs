/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::fmt::Write;

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

pub(crate) fn build_count_sql<T>(table_name: &str, where_clause: &str, opt: Option<&crate::db::Opt<T>>) -> String {
    let table_name = opt.and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table_name);
    let with_deleted_at = opt.and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);
    let mut sql = if where_clause.trim().is_empty() {
        if with_deleted_at {
            format!("SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL", table_name)
        } else {
            format!("SELECT COUNT(*) FROM {}", table_name)
        }
    } else if with_deleted_at {
        format!("SELECT COUNT(*) FROM {} WHERE ({}) AND deleted_at IS NULL", table_name, where_clause)
    } else {
        format!("SELECT COUNT(*) FROM {} WHERE {}", table_name, where_clause)
    };

    if let Some(tail_query) = opt.and_then(|o| o.tail_query.as_ref()) {
        sql.push(' ');
        sql.push_str(tail_query);
    }

    sql
}
pub(crate) fn build_insert_sql<T>(table_name: &str, columns: &str, opt: Option<&crate::db::Opt<T>>) -> String {
    let table_name = opt.and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table_name);

    let mut count = 0;
    let mut in_parens = 0;
    for c in columns.chars() {
        match c {
            '(' => in_parens += 1,
            ')' => in_parens -= 1,
            ',' if in_parens == 0 => count += 1,
            _ => {}
        }
    }
    count += 1; // last column

    let mut placeholders = String::with_capacity(count * 4);
    for i in 1..=count {
        if i > 1 {
            placeholders.push_str(", ");
        }

        write!(placeholders, "${}", i).unwrap();
    }
    format!("INSERT INTO {} ({}) VALUES ({})", table_name, columns, placeholders)
}
