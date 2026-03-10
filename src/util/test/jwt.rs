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
use crate::time;

#[test]
fn test_jwt_encode_decode() {
    let sub = "1234567890".to_string();
    let iss = "rmod".to_string();
    let duration = time::to_delta("1d");
    let secret = "secret";

    let token = encode(sub.clone(), iss.clone(), secret, duration);
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
    let duration = time::to_delta("1d");
    let secret = "secret";
    let wrong_secret = "wrong_secret";

    let token = encode(sub, iss, secret, duration);
    let result = decode(&token, wrong_secret);

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "invalid signature");
}

#[test]
fn test_jwt_expired() {
    let sub = "1234567890".to_string();
    let iss = "rmod".to_string();
    let duration = time::to_delta("-10s");
    let secret = "secret";

    let token = encode(sub, iss, secret, duration);
    let result = decode(&token, secret);

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "token expired");
}
