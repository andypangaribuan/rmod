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
    if let Some(full_query) = opt.and_then(|o| o.full_query.as_ref()) {
        if let Some(new_table) = opt.and_then(|o| o.table_name.as_ref()) {
            return replace_table_name(full_query, table_name, new_table);
        }
        return full_query.to_string();
    }

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
    if let Some(full_query) = opt.and_then(|o| o.full_query.as_ref()) {
        if let Some(new_table) = opt.and_then(|o| o.table_name.as_ref()) {
            return replace_table_name(full_query, table_name, new_table);
        }
        return full_query.to_string();
    }

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

pub(crate) fn build_custom_select_sql<T>(
    table_name: &str,
    select_clause: &str,
    where_clause: &str,
    opt: Option<&crate::db::Opt<T>>,
) -> String {
    if let Some(full_query) = opt.and_then(|o| o.full_query.as_ref()) {
        if let Some(new_table) = opt.and_then(|o| o.table_name.as_ref()) {
            return replace_table_name(full_query, table_name, new_table);
        }
        return full_query.to_string();
    }

    let table_name = opt.and_then(|o| o.table_name.as_ref()).map(|s| s.as_str()).unwrap_or(table_name);
    let with_deleted_at = opt.and_then(|o| o.with_deleted_at).unwrap_or_else(crate::store::get_db_with_deleted_at);
    let mut sql = if where_clause.trim().is_empty() {
        if with_deleted_at {
            format!("SELECT {} FROM {} WHERE deleted_at IS NULL", select_clause, table_name)
        } else {
            format!("SELECT {} FROM {}", select_clause, table_name)
        }
    } else if with_deleted_at {
        format!("SELECT {} FROM {} WHERE ({}) AND deleted_at IS NULL", select_clause, table_name, where_clause)
    } else {
        format!("SELECT {} FROM {} WHERE {}", select_clause, table_name, where_clause)
    };

    if let Some(tail_query) = opt.and_then(|o| o.tail_query.as_ref()) {
        sql.push(' ');
        sql.push_str(tail_query);
    }

    sql
}
pub(crate) fn build_insert_sql<T>(table_name: &str, columns: &str, opt: Option<&crate::db::Opt<T>>) -> String {
    if let Some(full_query) = opt.and_then(|o| o.full_query.as_ref()) {
        if let Some(new_table) = opt.and_then(|o| o.table_name.as_ref()) {
            return replace_table_name(full_query, table_name, new_table);
        }
        return full_query.to_string();
    }

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

pub(crate) async fn log_db_exec<F, T, E>(sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_db_query(sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_db_query(sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(crate) async fn log_tx_db_exec<F, T, E>(tx: &super::Tx, sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_tx_db_query(&tx.id, tx.key.as_deref(), sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_tx_db_query(&tx.id, tx.key.as_deref(), sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(crate) async fn log_db_update<F, T, E>(sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_db_update(sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_db_update(sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(crate) async fn log_tx_db_update<F, T, E>(tx: &super::Tx, sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_db_tx_update(&tx.id, tx.key.as_deref(), sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_db_tx_update(&tx.id, tx.key.as_deref(), sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(crate) async fn log_db_execute<F, T, E>(sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_db_execute(sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_db_execute(sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(crate) async fn log_tx_db_execute<F, T, E>(tx: &super::Tx, sql: &str, args: Option<&[String]>, fut: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    T: std::fmt::Debug,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let res = fut.await;
    let duration_ms = start.elapsed().as_millis() as i32;

    match &res {
        Ok(val) => {
            let res_str = format!("{:?}", val);
            let res_truncated = if res_str.len() > 100_000 { format!("{}... [TRUNCATED]", &res_str[..100_000]) } else { res_str };
            crate::clog::log_db_tx_execute(&tx.id, tx.key.as_deref(), sql, args, Some(&res_truncated), duration_ms, 200, None, None);
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = format!("{}", bt);
            let stacktrace = if !bt_str.trim().is_empty() { Some(bt_str.as_str()) } else { None };
            crate::clog::log_db_tx_execute(&tx.id, tx.key.as_deref(), sql, args, None, duration_ms, 500, Some(&e.to_string()), stacktrace);
        }
    }

    res
}

pub(super) fn replace_table_name(query: &str, original: &str, new_table: &str) -> String {
    let mut result = String::with_capacity(query.len() + new_table.len());
    let mut last_end = 0;

    for (start, _) in query.match_indices(original) {
        let end = start + original.len();

        let is_valid_start = if start == 0 {
            true
        } else {
            let ch = query[..start].chars().next_back().unwrap();
            !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
        };

        let is_valid_end = if end == query.len() {
            true
        } else {
            let ch = query[end..].chars().next().unwrap();
            !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
        };

        if is_valid_start && is_valid_end {
            result.push_str(&query[last_end..start]);
            result.push_str(new_table);
            last_end = end;
        }
    }
    result.push_str(&query[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct WithDebug {
        a: i32,
    }

    struct WithoutDebug {
        _a: i32,
    }

    #[test]
    fn test_dbg_specialization() {
        let wd = WithDebug { a: 42 };
        let wod = WithoutDebug { _a: 100 };
        let opt_wd = Some(WithDebug { a: 42 });

        let res_wd = DbgWrap(&wd).opt_fmt();
        let res_wod = DbgWrap(&wod).opt_fmt();
        let res_opt_wd = DbgWrap(&opt_wd).opt_fmt();

        assert_eq!(res_wd, Some("WithDebug { a: 42 }".to_string()));
        assert_eq!(res_wod, None);
        assert_eq!(res_opt_wd, Some("Some(WithDebug { a: 42 })".to_string()));
    }
}
