/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};

const KEY_LEN: usize = 32; // AES-256
const IV_LEN: usize = 12; // GCM standard nonce size

/// Encrypts data using AES-256-GCM.
/// Key and IV (nonce) are expected to be base64 encoded strings.
/// Output is a base64 encoded string containing [ciphertext | tag].
pub fn encrypt(data: &[u8], key_b64: &str, iv_b64: &str) -> Result<String, String> {
    let key_bytes = STANDARD.decode(key_b64).map_err(|e| format!("invalid key base64: {}", e))?;
    let iv_bytes = STANDARD.decode(iv_b64).map_err(|e| format!("invalid iv base64: {}", e))?;
    _encrypt(data, &key_bytes, &iv_bytes)
}

/// Encrypts data using AES-256-GCM.
/// Key and IV (nonce) are expected to be raw strings.
/// Output is a base64 encoded string containing [ciphertext | tag].
pub fn encrypt_raw(data: &[u8], key: &str, iv: &str) -> Result<String, String> {
    _encrypt(data, key.as_bytes(), iv.as_bytes())
}

fn _encrypt(data: &[u8], key_bytes: &[u8], iv_bytes: &[u8]) -> Result<String, String> {
    if key_bytes.len() != KEY_LEN {
        return Err(format!("invalid key length: expected {} bytes, got {}", KEY_LEN, key_bytes.len()));
    }

    if iv_bytes.len() != IV_LEN && iv_bytes.len() != 16 {
        return Err(format!("invalid iv length: expected {} or 16 bytes, got {}", IV_LEN, iv_bytes.len()));
    }

    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| format!("cipher initialization failed: {:?}", e))?;
    let nonce = Nonce::from_slice(iv_bytes);

    let ciphertext = cipher.encrypt(nonce, Payload { msg: data, aad: &[] }).map_err(|e| format!("encryption failed: {:?}", e))?;

    Ok(STANDARD.encode(ciphertext))
}

/// Decrypts data using AES-256-GCM.
/// Key and IV (nonce) are expected to be base64 encoded strings.
/// Input is a base64 encoded string containing [ciphertext | tag].
pub fn decrypt(encoded_data: &str, key_b64: &str, iv_b64: &str) -> Result<Vec<u8>, String> {
    let key_bytes = STANDARD.decode(key_b64).map_err(|e| format!("invalid key base64: {}", e))?;
    let iv_bytes = STANDARD.decode(iv_b64).map_err(|e| format!("invalid iv base64: {}", e))?;
    _decrypt(encoded_data, &key_bytes, &iv_bytes)
}

/// Decrypts data using AES-256-GCM.
/// Key and IV (nonce) are expected to be raw strings.
/// Input is a base64 encoded string containing [ciphertext | tag].
pub fn decrypt_raw(encoded_data: &str, key: &str, iv: &str) -> Result<Vec<u8>, String> {
    _decrypt(encoded_data, key.as_bytes(), iv.as_bytes())
}

fn _decrypt(encoded_data: &str, key_bytes: &[u8], iv_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let encrypted_data = STANDARD.decode(encoded_data).map_err(|e| format!("invalid data base64: {}", e))?;

    if key_bytes.len() != KEY_LEN {
        return Err(format!("invalid key length: expected {} bytes, got {}", KEY_LEN, key_bytes.len()));
    }

    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| format!("cipher initialization failed: {:?}", e))?;
    let nonce = Nonce::from_slice(iv_bytes);

    let decrypted_data =
        cipher.decrypt(nonce, Payload { msg: &encrypted_data, aad: &[] }).map_err(|e| format!("decryption failed: {:?}", e))?;

    Ok(decrypted_data)
}

pub fn argon2id_hash(password: &str, salt: Option<&[u8]>) -> Result<String, String> {
    let params = Params::new(30 * 1024, 6, 3, Some(32)).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let salt_string = match salt {
        Some(s) => SaltString::encode_b64(s).map_err(|e| e.to_string())?,
        None => SaltString::generate(&mut OsRng),
    };

    let password_hash = argon2.hash_password(password.as_bytes(), &salt_string).map_err(|e| e.to_string())?;

    Ok(password_hash.to_string())
}

pub fn argon2id_match(password: &str, encoded_hash: &str) -> Result<bool, String> {
    let hash = PasswordHash::new(encoded_hash).map_err(|e| e.to_string())?;

    match Argon2::default().verify_password(password.as_bytes(), &hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::{SystemTime, UNIX_EPOCH};

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

        // timestamp into nanoseconds precision
        let timestamp_nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos();
        println!("Timestamp (nanoseconds): {}", timestamp_nanos);

        let now = Utc::now();
        println!("Timestamp (human): {}", now.format("%Y-%m-%dT%H:%M:%S%.9f%:z"));
        let now_again = Utc::now();
        println!("Timestamp (human 2): {}", now_again.to_rfc3339());
        println!("Timestamp (human 2): {}", now_again.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        println!("Timestamp (human 2): {}", now_again.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
        println!("Timestamp (human 2): {}", now_again.to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
        println!("Timestamp (human 3): {}", now_again.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true));

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
}
