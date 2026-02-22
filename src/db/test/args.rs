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
fn test_db_args_with_opt() {
    let opt = args_opt::<()>("ORDER BY id");
    let args = args!("val1", opt);
    assert!(args.opt.is_some());
    assert_eq!(args.opt.as_ref().unwrap().tail_query, Some("ORDER BY id".to_string()));
    assert!(!args.is_force_rw());
}

#[test]
fn test_db_args_with_opt_rw() {
    let opt = args_opt_rw::<()>("ORDER BY id");
    let args = args!("val1", opt);
    assert!(args.is_force_rw());
    assert_eq!(args.opt.as_ref().unwrap().tail_query, Some("ORDER BY id".to_string()));
}

#[test]
fn test_opt_builder() {
    let opt = Opt::<i32>::new().with_tail_query("LIMIT 10").with_force_rw().with_validate(|res| res.is_some());

    assert_eq!(opt.tail_query, Some("LIMIT 10".to_string()));
    assert_eq!(opt.force_rw, Some(true));
    assert!(opt.validate.is_some());

    let validator = opt.validate.as_ref().unwrap();
    assert!(validator(&Some(1)));
    assert!(!validator(&None));
}

#[test]
fn test_opt_validate_all() {
    let opt = Opt::<i32>::new().with_validate_all(|res| !res.is_empty());

    assert!(opt.validate_all.is_some());
    let validator = opt.validate_all.as_ref().unwrap();
    assert!(validator(&vec![1, 2, 3]));
    assert!(!validator(&vec![]));
}

#[test]
fn test_pg_args_reproducibility() {
    // Testing that build_inner can be called multiple times
    // (critical for retry-on-master logic)
    let args: PgArgs<()> = args!(1, "two", 3.0);

    // First build
    let _inner1 = args.build_inner();

    // Second build (should not panic or lose data)
    let _inner2 = args.build_inner();

    assert_eq!(args.collectors.len(), 3);
}
