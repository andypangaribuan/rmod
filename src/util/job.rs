/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::time::Duration;
use tokio::time::{MissedTickBehavior, interval};

pub struct JobScheduler {
    jobs: Vec<Job>,
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}

struct Job {
    pub duration: Duration,
    pub handler: fn(),
}

impl JobScheduler {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add(&mut self, duration: &str, handler: fn()) {
        let duration = parse_duration(duration);
        self.jobs.push(Job { duration, handler });
    }

    pub fn start(self) {
        for job in self.jobs {
            tokio::spawn(async move {
                let mut interval = interval(job.duration);
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    interval.tick().await;
                    (job.handler)();
                }
            });
        }
    }
}

fn parse_duration(duration: &str) -> Duration {
    let mut val = duration.to_string();
    let mut unit = "s";

    if val.ends_with("ms") {
        unit = "ms";
        val = val[..val.len() - 2].to_string();
    } else if val.ends_with('s') {
        unit = "s";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('m') {
        unit = "m";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('h') {
        unit = "h";
        val = val[..val.len() - 1].to_string();
    } else if val.ends_with('d') {
        unit = "d";
        val = val[..val.len() - 1].to_string();
    }

    let val = val.parse::<u64>().unwrap_or(0);

    match unit {
        "ms" => Duration::from_millis(val),
        "s" => Duration::from_secs(val),
        "m" => Duration::from_secs(val * 60),
        "h" => Duration::from_secs(val * 3600),
        "d" => Duration::from_secs(val * 86400),
        _ => Duration::from_secs(val),
    }
}
