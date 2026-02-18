/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::fct;

#[test]
fn test_addition() {
    let a = 0.1;
    let b = 0.2;
    let result = a + b;
    println!("a + b: {}", result);

    let x = fct!(0.1);
    let y = fct!(0.2);
    let z = x + y;
    println!("x + y: {}", z);
    // assert_eq!(formatted, "2024-02-03 07:38:42");
}
