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
#[path = "test/jwt.rs"]
mod tests;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{TimeDelta, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;
static ENCODED_HEADER: OnceLock<String> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub iat: u32,
    pub exp: u32,
}

pub fn encode(sub: String, iss: String, secret: &str, exp_delta: TimeDelta) -> String {
    let timenow = Utc::now();
    let iat = timenow.timestamp() as u32;
    let exp = (timenow + exp_delta).timestamp() as u32;

    let claims = Claims { sub, iss, iat, exp };

    let encoded_header = ENCODED_HEADER.get_or_init(|| {
        let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
        let header_json = serde_json::to_string(&header).unwrap();
        URL_SAFE_NO_PAD.encode(header_json.as_bytes())
    });

    let payload_json = serde_json::to_string(&claims).unwrap();
    let encoded_payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let data = format!("{}.{}", encoded_header, encoded_payload);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    let signature = URL_SAFE_NO_PAD.encode(result.into_bytes());

    format!("{}.{}", data, signature)
}

pub fn decode(token: &str, secret: &str) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("invalid token format".to_string());
    }

    let last_dot = token.rfind('.').ok_or("invalid token format".to_string())?;
    let data = &token[..last_dot];
    let signature = URL_SAFE_NO_PAD.decode(parts[2]).map_err(|e| format!("invalid signature encoding: {}", e))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());

    if mac.verify_slice(&signature).is_err() {
        return Err("invalid signature".to_string());
    }

    let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).map_err(|e| format!("invalid payload encoding: {}", e))?;
    let payload: Claims = serde_json::from_slice(&payload_bytes).map_err(|e| format!("failed to parse payload: {}", e))?;

    if payload.exp < Utc::now().timestamp() as u32 {
        return Err("token expired".to_string());
    }

    Ok(payload)
}

pub fn unsafe_decode(token: &str) -> Option<Claims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let payload: Claims = serde_json::from_slice(&payload_bytes).ok()?;

    Some(payload)
}
