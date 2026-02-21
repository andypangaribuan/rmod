/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub use tokio::time::Duration;

#[cfg(test)]
#[path = "test/time.rs"]
mod tests;

pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
