/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use futures_util::future::BoxFuture;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{MissedTickBehavior, interval};

struct Job {
    duration: Duration,
    handler: fn() -> BoxFuture<'static, ()>,
    is_every: bool,
    zero_start: bool,
}

static JOBS: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

fn get_jobs() -> &'static Mutex<Vec<Job>> {
    JOBS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn add(duration: &str, handler: fn() -> BoxFuture<'static, ()>, is_every: bool, zero_start: bool) {
    let mut jobs = get_jobs().lock().unwrap();
    let duration = crate::util::conv::to_duration(duration);
    jobs.push(Job { duration, handler, is_every, zero_start });
}

pub fn start() {
    let mut jobs_lock = get_jobs().lock().unwrap();
    let jobs = std::mem::take(&mut *jobs_lock);

    for job in jobs {
        tokio::spawn(async move {
            let duration_ms = job.duration.as_millis();

            if job.zero_start && duration_ms > 0 {
                let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
                let next_ms = ((now_ms / duration_ms) + 1) * duration_ms;
                let delay_ms = (next_ms - now_ms) as u64;
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }

            if job.is_every {
                let mut interval = interval(job.duration);
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    interval.tick().await;
                    (job.handler)().await;
                }
            } else {
                loop {
                    (job.handler)().await;
                    tokio::time::sleep(job.duration).await;
                }
            }
        });
    }
}
