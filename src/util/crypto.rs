/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use aes::Aes256;
use base64::{Engine, engine::general_purpose::STANDARD};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use pbkdf2::pbkdf2_hmac;
use rand::RngExt;
use sha2::Sha256;

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

const KEY_LEN: usize = 32; // AES-256
const IV_LEN: usize = 16; // AES block size
const ITERATIONS: u32 = 100_000;

/// Encrypts data using AES-256-CBC with PKCS7 padding.
/// Key is derived from passphrase using PBKDF2-SHA256 with provided salt.
/// A random IV is generated and prepended to the ciphertext.
pub fn encrypt(data: &[u8], passphrase: &str, salt: &[u8]) -> Result<String, String> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, ITERATIONS, &mut key);

    let mut iv = [0u8; IV_LEN];
    rand::rng().fill(&mut iv);

    let ciphertext = {
        let mut buffer = vec![0u8; data.len() + IV_LEN]; // Sufficient space for PKCS7 padding
        buffer[..data.len()].copy_from_slice(data);

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len()).map_err(|e| format!("encryption failed: {:?}", e))?.to_vec()
    };

    let mut result = Vec::with_capacity(IV_LEN + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(result))
}

/// Decrypts data using AES-256-CBC with PKCS7 padding.
/// Data is expected to be base64 encoded string containing [IV (16 bytes) | ciphertext].
pub fn decrypt(encoded_data: &str, passphrase: &str, salt: &[u8]) -> Result<Vec<u8>, String> {
    let encrypted_data = STANDARD.decode(encoded_data).map_err(|e| format!("invalid base64: {}", e))?;

    if encrypted_data.len() < IV_LEN {
        return Err("data too short".to_string());
    }

    let (iv, ciphertext) = encrypted_data.split_at(IV_LEN);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, ITERATIONS, &mut key);

    let mut buffer = ciphertext.to_vec();
    let cipher = Aes256CbcDec::new(&key.into(), iv.into());
    let decrypted_data = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer).map_err(|e| format!("decryption failed: {:?}", e))?;

    Ok(decrypted_data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_works() {
        let passphrase = "very-secure-secret-100";
        let salt = b"some salt";
        let data = b"hello world";

        let encrypted = encrypt(data, passphrase, salt).unwrap();
        let decrypted = decrypt(&encrypted, passphrase, salt).unwrap();

        let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8 sequence");

        println!("encrypted: {}", encrypted);
        println!("decrypted: {}", decrypted_str);

        assert_eq!(data, decrypted_str.as_bytes());
    }
}
