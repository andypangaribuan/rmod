/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::FuseRContext;
use axum::extract::ConnectInfo;
use std::net::IpAddr;

impl FuseRContext {
    pub fn client_ip(&self) -> String {
        let headers = self.req.headers();

        // 1. X-Client-IP
        if let Some(ip) = headers.get("x-client-ip").and_then(|v| v.to_str().ok()) {
            return ip.trim().to_string();
        }

        // 2. X-Original-Forwarded-For
        if let Some(val) = headers.get("x-original-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(ip) = self.retrieve_forwarded_ip(val) {
                return ip;
            }
        }

        // 3. X-Forwarded-For
        if let Some(val) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(ip) = self.retrieve_forwarded_ip(val) {
                return ip;
            }
        }

        // 4. Special headers
        let special_headers = ["cf-connecting-ip", "fastly-client-ip", "true-client-ip", "x-real-ip"];
        for header in special_headers {
            if let Some(ip) = headers.get(header).and_then(|v| v.to_str().ok()) {
                return ip.trim().to_string();
            }
        }

        // 5. Other forwarded headers
        let forwarded_headers = ["x-forwarded", "forwarded-for", "forwarded"];
        for header in forwarded_headers {
            if let Some(val) = headers.get(header).and_then(|v| v.to_str().ok()) {
                if let Some(ip) = self.retrieve_forwarded_ip(val) {
                    return ip;
                }
            }
        }

        // 6. Remote Address fallback
        self.req
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip().to_string())
            .unwrap_or_else(|| "".to_string())
    }

    fn retrieve_forwarded_ip(&self, header_val: &str) -> Option<String> {
        for address in header_val.split(',') {
            let address = address.trim();
            if !address.is_empty() {
                if let Ok(ip) = address.parse::<IpAddr>() {
                    if !self.is_private_ip(ip) {
                        return Some(address.to_string());
                    }
                }
            }
        }
        None
    }

    fn is_private_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
            IpAddr::V6(ip) => ip.is_loopback() || (ip.segments()[0] & 0xfe00) == 0xfc00 || (ip.segments()[0] & 0xffc0) == 0xfe80,
        }
    }
}
