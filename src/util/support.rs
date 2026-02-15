/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        println!("{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z"), format!($($arg)*));
    };
}
