/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::*;

#[tokio::test]
async fn test_sleep() {
    let start = std::time::Instant::now();
    let duration = Duration::from_millis(100);
    sleep(duration).await;
    let elapsed = start.elapsed();
    assert!(elapsed >= duration);
}
