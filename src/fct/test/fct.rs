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
    let a1: f64 = 0.1;
    let a2: f64 = 0.2;
    let a12 = a1 + a2;
    println!("f64(0.1) + f64(0.2): {}", a12);

    let b1 = fct!(0.1);
    let b2 = fct!(0.2);
    let b12 = b1 + b2;
    println!("fct(0.1) + fct(0.2): {}", b12);
    assert_eq!(b12, fct!(0.3));

    let c1: i32 = 1;
    let c2 = fct!(0.2);
    let c12 = c1 + c2;
    println!("i32(1) + fct(0.2): {}", c12);
    assert_eq!(c12, fct!(1.2));

    let d1: u32 = 1;
    let d2 = fct!(0.2);
    let d12 = d1 + d2;
    println!("u32(1) + fct(0.2): {}", d12);
    assert_eq!(d12, fct!(1.2));

    let e1: i64 = 1;
    let e2 = fct!(0.2);
    let e12 = e1 + e2;
    println!("i64(1) + fct(0.2): {}", e12);
    assert_eq!(e12, fct!(1.2));

    let f1: u64 = 1;
    let f2 = fct!(0.2);
    let f12 = f1 + f2;
    println!("u64(1) + fct(0.2): {}", f12);
    assert_eq!(f12, fct!(1.2));

    let g1: i128 = 1;
    let g2 = fct!(0.2);
    let g12 = g1 + g2;
    println!("i128(1) + fct(0.2): {}", g12);
    assert_eq!(g12, fct!(1.2));

    let h1: u128 = 1;
    let h2 = fct!(0.2);
    let h12 = h1 + h2;
    println!("u128(1) + fct(0.2): {}", h12);
    assert_eq!(h12, fct!(1.2));

    let i1: isize = 1;
    let i2 = fct!(0.2);
    let i12 = i1 + i2;
    println!("isize(1) + fct(0.2): {}", i12);
    assert_eq!(i12, fct!(1.2));

    let j1: usize = 1;
    let j2 = fct!(0.2);
    let j12 = j1 + j2;
    println!("usize(1) + fct(0.2): {}", j12);
    assert_eq!(j12, fct!(1.2));

    let k1: f32 = 0.1;
    let k2 = fct!(0.2);
    let k12 = k1 + k2;
    println!("f32(0.1) + fct(0.2): {}", k12);
    assert_eq!(k12, fct!(0.3));

    let l1: f64 = 0.1;
    let l2 = fct!(0.2);
    let l12 = l1 + l2;
    println!("f64(0.1) + fct(0.2): {}", l12);
    assert_eq!(l12, fct!(0.3));
}

#[test]
fn test_to_str() {
    let v1 = fct!(3.30000000000000);
    assert_eq!(v1.to_str(0), "3.3");
    assert_eq!(v1.to_str(6), "3.300000");
    assert_eq!(v1.to_str(7), "3.3000000");

    let v2 = fct!(3.30000090000000);
    assert_eq!(v2.to_str(0), "3.3000009");
    assert_eq!(v2.to_str(6), "3.300000");
    assert_eq!(v2.to_str(7), "3.3000009");

    let v3 = fct!(3);
    assert_eq!(v3.to_str(0), "3");
    assert_eq!(v3.to_str(2), "3.00");
}
