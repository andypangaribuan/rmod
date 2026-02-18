/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/crypto.rs"]
mod tests;

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
