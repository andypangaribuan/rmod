/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub fn healthcheck(port: i16) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--healthcheck" {
        let addr = format!("127.0.0.1:{}", port);
        let mut stream = std::net::TcpStream::connect(addr).unwrap_or_else(|_| std::process::exit(1));
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap_or_else(|_| std::process::exit(1));
        stream.set_write_timeout(Some(std::time::Duration::from_secs(3))).unwrap_or_else(|_| std::process::exit(1));

        use std::io::{Read, Write};
        let request = "GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(request.as_bytes()).unwrap_or_else(|_| std::process::exit(1));

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap_or_else(|_| std::process::exit(1));

        if response.contains("200 OK") {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }
}

pub fn grpc_healthcheck(port: i16) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--healthcheck" {
        use tonic_health::pb::HealthCheckRequest;
        use tonic_health::pb::health_client::HealthClient;

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async {
            let addr = format!("http://127.0.0.1:{}", port);
            let channel = match tonic::transport::Endpoint::from_shared(addr) {
                Ok(endpoint) => match endpoint.connect().await {
                    Ok(channel) => channel,
                    Err(_) => std::process::exit(1),
                },
                Err(_) => std::process::exit(1),
            };

            let mut client = HealthClient::new(channel);

            let request = tonic::Request::new(HealthCheckRequest { service: "".to_string() });

            match client.check(request).await {
                Ok(resp) => {
                    use tonic_health::pb::health_check_response::ServingStatus;
                    if resp.into_inner().status == ServingStatus::Serving as i32 {
                        std::process::exit(0);
                    } else {
                        std::process::exit(1);
                    }
                }
                Err(_) => std::process::exit(1),
            }
        });
    }
}
