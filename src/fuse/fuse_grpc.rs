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

pub async fn grpc<S>(addr: &str, service: S)
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
{
    let addr: SocketAddr = addr.parse().unwrap();
    let mut shutdown_rx = crate::util::lifecycle::subscribe();

    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .unwrap();

    crate::util::lifecycle::wait().await;
}
