/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::*;

#[test]
fn test_log_macro() {
    crate::log!("test log message");
    crate::log!("test log with arg: {}", 123);
}

#[test]
fn test_collect_unique() {
    let items = vec![1, 2, 2, 3, 1, 4];
    let unique = collect_unique(items, |x| x);
    assert_eq!(unique, vec![1, 2, 3, 4]);

    struct Item {
        id: i32,
        _name: &'static str,
    }
    let items = vec![Item { id: 1, _name: "a" }, Item { id: 2, _name: "b" }, Item { id: 1, _name: "c" }];
    let unique_ids = collect_unique(items, |x| x.id);
    assert_eq!(unique_ids, vec![1, 2]);
}

#[test]
fn test_have_in() {
    let list = vec![1, 2, 3];
    assert!(have_in(2, list.clone()));
    assert!(!have_in(4, list));

    let list = vec!["a".to_string(), "b".to_string()];
    assert!(have_in("a".to_string(), list.clone()));
    assert!(!have_in("c".to_string(), list));
}
