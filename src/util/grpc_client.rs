/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::tonic::transport::{Channel, Endpoint};
use std::time::Duration;

pub async fn connect(url: &str) -> Result<Channel, crate::tonic::transport::Error> {
    Endpoint::from_shared(url.to_string())?.connect().await
}

pub async fn connect_with_timeout(url: &str, timeout: Duration) -> Result<Channel, crate::tonic::transport::Error> {
    Endpoint::from_shared(url.to_string())?.timeout(timeout).connect().await
}
