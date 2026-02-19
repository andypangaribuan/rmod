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
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;

#[test]
fn test_it_works() {
    let key_b64 = STANDARD.encode("this-is-a-32-byte-secret-key-123");
    let iv_b64 = STANDARD.encode("12-byte-iv--");
    let data = b"hello world";

    let encrypted = encrypt(data, &key_b64, &iv_b64).unwrap();
    let decrypted = decrypt(&encrypted, &key_b64, &iv_b64).unwrap();

    let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8 sequence");

    println!("encrypted: {}", encrypted);
    println!("decrypted: {}", decrypted_str);

    let timenow = Utc::now();
    println!("Timestamp (secs)   : {}", timenow.timestamp());
    println!("Timestamp (millis) : {}", timenow.timestamp_millis());
    println!("Timestamp (nanos)  : {}", timenow.timestamp_nanos_opt().unwrap());

    println!("Timestamp (second) : {}", timenow.format("%Y-%m-%dT%H:%M:%S%.9f%:z"));
    println!("Timestamp (rfc3339): {}", timenow.to_rfc3339());
    println!("Timestamp (secs)   : {}", timenow.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    println!("Timestamp (millis) : {}", timenow.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
    println!("Timestamp (micros) : {}", timenow.to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
    println!("Timestamp (nanos)  : {}", timenow.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true));

    assert_eq!(data, decrypted_str.as_bytes());
}

#[test]
fn test_raw_it_works() {
    let key = "this-is-a-32-byte-secret-key-123";
    let iv = "12-byte-iv--";
    let data = b"hello world";

    let encrypted = encrypt_raw(data, key, iv).unwrap();
    let decrypted = decrypt_raw(&encrypted, key, iv).unwrap();

    let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8 sequence");

    println!("encrypted: {}", encrypted);
    println!("decrypted: {}", decrypted_str);

    assert_eq!(data, decrypted_str.as_bytes());
}

#[test]
fn test_argon2id() {
    let password = "strong-password";
    let hash = argon2id_hash(password, None).unwrap();
    println!("hash: {}", hash);

    assert!(argon2id_match(password, &hash).unwrap());
    assert!(!argon2id_match("wrong-password", &hash).unwrap());
}

#[test]
fn test_argon2id_with_salt() {
    let password = "my-secret-password";
    let salt = b"some-salt-123456";
    let hash = argon2id_hash(password, Some(salt)).unwrap();
    println!("hash with salt: {}", hash);

    assert!(argon2id_match(password, &hash).unwrap());
    assert!(hash.contains("$argon2id$v=19$m=30720,t=6,p=3$"));
}

#[test]
fn test_encrypt_iv_length() {
    let key = "12345678901234567890123456789012";
    let iv_12 = "123456789012";
    let iv_16 = "1234567890123456";

    let res_12 = encrypt_raw(b"test", key, iv_12);
    assert!(res_12.is_ok());

    // This is expected to fail or panic
    let res_16 = encrypt_raw(b"test", key, iv_16);
    assert!(res_16.is_err());
}
