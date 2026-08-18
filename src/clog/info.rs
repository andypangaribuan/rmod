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
