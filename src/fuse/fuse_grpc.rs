/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use prost;
use std::net::SocketAddr;
pub use tonic;
use tonic::transport::Server;
pub use tonic_health;

pub async fn grpc<S, F>(addr: &str, service: S, on_start: Option<F>)
where
    S: tonic::codegen::Service<
            tonic::codegen::http::Request<tonic::body::BoxBody>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    F: FnOnce(),
{
    let addr: SocketAddr = addr.parse().unwrap_or_else(|e| {
        tracing::error!("Failed to parse gRPC bind address '{}': {}", addr, e);
        std::process::exit(1);
    });
    let mut shutdown_rx = crate::util::lifecycle::subscribe();

    if let Some(f) = on_start {
        f();
    }

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<S>().await;
    health_reporter.set_service_status("", tonic_health::ServingStatus::Serving).await;

    if let Err(e) = Server::builder()
        .add_service(health_service)
        .add_service(service)
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
    {
        tracing::error!("gRPC server failed: {}", e);
        std::process::exit(1);
    }

    crate::util::lifecycle::wait().await;
}
