/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tokio::time::{MissedTickBehavior, interval};

struct Job {
    duration: Duration,
    handler: fn(),
    is_every: bool,
}

static JOBS: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

fn get_jobs() -> &'static Mutex<Vec<Job>> {
    JOBS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn add(duration: &str, handler: fn(), is_every: bool) {
    let mut jobs = get_jobs().lock().unwrap();
    let duration = crate::util::conv::to_duration(duration);
    jobs.push(Job { duration, handler, is_every });
}

pub fn start() {
    let mut jobs_lock = get_jobs().lock().unwrap();
    let jobs = std::mem::take(&mut *jobs_lock);

    for job in jobs {
        if job.is_every {
            tokio::spawn(async move {
                let mut interval = interval(job.duration);
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    interval.tick().await;
                    (job.handler)();
                }
            });
        } else {
            tokio::spawn(async move {
                tokio::time::sleep(job.duration).await;
                (job.handler)();
            });
        }
    }
}
