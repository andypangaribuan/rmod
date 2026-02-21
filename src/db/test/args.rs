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
    let opt = args_opt("ORDER BY id");
    let args = args!("val1", opt);
    assert!(args.opt.is_some());
    assert_eq!(args.opt.as_ref().unwrap().end_query, Some("ORDER BY id".to_string()));
    assert!(!args.is_force_rw());
}

#[test]
fn test_db_args_with_opt_rw() {
    let opt = args_opt_rw("ORDER BY id");
    let args = args!("val1", opt);
    assert!(args.is_force_rw());
    assert_eq!(args.opt.as_ref().unwrap().end_query, Some("ORDER BY id".to_string()));
}
