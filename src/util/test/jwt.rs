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
use chrono::TimeDelta;

#[test]
fn test_jwt_encode_decode() {
    let sub = "1234567890".to_string();
    let iss = "rmod".to_string();
    let exp_delta = TimeDelta::hours(24);
    let secret = "secret";

    let token = encode(sub.clone(), iss.clone(), secret, exp_delta);
    let decoded = decode(&token, secret).unwrap();

    println!("token: {}", token);
    println!("decoded: {:#?}", decoded);
    assert_eq!(sub, decoded.sub);
    assert!(decoded.exp > decoded.iat);
}

#[test]
fn test_jwt_invalid_secret() {
    let sub = "1234567890".to_string();
    let iss = "rmod".to_string();
    let exp_delta = TimeDelta::hours(24);
    let secret = "secret";
    let wrong_secret = "wrong_secret";

    let token = encode(sub, iss, secret, exp_delta);
    let result = decode(&token, wrong_secret);

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "invalid signature");
}

#[test]
fn test_jwt_expired() {
    let sub = "1234567890".to_string();
    let iss = "rmod".to_string();
    let exp_delta = TimeDelta::seconds(-10); // 10 seconds ago
    let secret = "secret";

    let token = encode(sub, iss, secret, exp_delta);
    let result = decode(&token, secret);

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "token expired");
}
