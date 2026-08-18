/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::util::env;
use local_ip_address;

pub fn pod_name() -> String {
    let pod_name = env::string_or("POD_NAME", "");
    let pod_namespace = env::string_or("POD_NAMESPACE", "");
    if !pod_name.is_empty() && !pod_namespace.is_empty() {
        return format!("{}.{}", pod_name, pod_namespace);
    }

    let hostname = env::string_or("HOSTNAME", "");
    if !hostname.is_empty() {
        return hostname;
    }

    let hostname = hostname::get().expect("");
    format!("{}", hostname.to_string_lossy())
}

pub fn pod_info() -> (String, String) {
    let mut pod_ip = env::string_or("POD_IP", "");
    let node_name = env::string_or("NODE_NAME", "");

    if pod_ip.is_empty() {
        pod_ip = local_ip_address::local_ip().expect("").to_string();
    }

    (pod_ip, node_name)
}

pub fn clean_stacktrace(bt_str: &str) -> String {
    let raw_text = bt_str.trim();
    if raw_text.is_empty() {
        return String::new();
    }

    let service_name_sub = crate::clog::get_config()
        .map(|c| c.service_name.replace('-', "_"))
        .unwrap_or_default();

    let mut frames: Vec<Vec<&str>> = Vec::new();
    let mut current_frame: Vec<&str> = Vec::new();

    for line in raw_text.lines() {
        let trimmed = line.trim_start();
        let is_frame_start = trimmed.find(':').map(|idx| {
            let prefix = trimmed[..idx].trim();
            !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit())
        }).unwrap_or(false);

        if is_frame_start {
            if !current_frame.is_empty() {
                frames.push(current_frame);
                current_frame = Vec::new();
            }
        }
        current_frame.push(line);
    }
    if !current_frame.is_empty() {
        frames.push(current_frame);
    }

    let mut filtered_frames: Vec<String> = Vec::new();
    let mut frame_index = 0;

    for frame in frames {
        let full_frame_text = frame.join("\n");
        let lower = full_frame_text.to_lowercase();

        let is_system_or_dep = lower.contains("/.cargo/registry/")
            || lower.contains("/.rustup/")
            || lower.contains("/rustc/")
            || lower.contains("std::")
            || lower.contains("core::")
            || lower.contains("alloc::")
            || lower.contains("backtrace_rs")
            || lower.contains("backtrace::")
            || lower.contains("tokio::")
            || lower.contains("axum::")
            || lower.contains("tower::")
            || lower.contains("hyper::")
            || lower.contains("hyper_util::")
            || lower.contains("futures_util::")
            || lower.contains("sqlx::")
            || lower.contains("___rust_try")
            || lower.contains("__pthread");

        let is_rmod = lower.contains("rmod") || lower.contains("/rmod/");
        let is_service = !service_name_sub.is_empty() && lower.contains(&service_name_sub);
        let is_app_src = (lower.contains("./src/") || lower.contains("/src/"))
            && !lower.contains("/.cargo/")
            && !lower.contains("/.rustup/")
            && !lower.contains("/rustc/");

        if is_rmod || is_service || is_app_src || !is_system_or_dep {
            let mut frame_lines: Vec<String> = frame.iter().map(|s| s.to_string()).collect();
            if let Some(first_line) = frame_lines.first_mut() {
                let trimmed = first_line.trim_start();
                if let Some(colon_pos) = trimmed.find(':') {
                    let rest = &trimmed[colon_pos + 1..];
                    *first_line = format!("{:4}:{}", frame_index, rest);
                }
            }
            filtered_frames.push(frame_lines.join("\n"));
            frame_index += 1;
        }
    }

    if filtered_frames.is_empty() {
        return raw_text.to_string();
    }

    filtered_frames.join("\n")
}
