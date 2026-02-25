/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};

type ShutdownCallback = Box<dyn FnOnce() + Send + 'static>;

static SHUTDOWN_DURATION: Lazy<Mutex<Option<Duration>>> = Lazy::new(|| Mutex::new(None));
static CALLBACKS: Lazy<Mutex<Vec<ShutdownCallback>>> = Lazy::new(|| Mutex::new(Vec::new()));
static STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn before_graceful_shutdown<F>(callbacks: Vec<F>)
where
    F: FnOnce() + Send + 'static,
{
    let mut guard = CALLBACKS.lock().unwrap();
    for cb in callbacks {
        guard.push(Box::new(cb));
    }
}

pub fn graceful_shutdown(wait_duration: Option<Duration>) {
    let mut guard = SHUTDOWN_DURATION.lock().unwrap();
    *guard = wait_duration;
}

pub fn start() {
    let mut started = STARTED.lock().unwrap();
    if *started {
        return;
    }
    *started = true;
    tokio::spawn(async move {
        let mut sig_int = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        let mut sig_term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = sig_int.recv() => {},
            _ = sig_term.recv() => {},
        };

        let cbs = {
            let mut guard = CALLBACKS.lock().unwrap();
            std::mem::take(&mut *guard)
        };
        for cb in cbs {
            tokio::spawn(async move {
                cb();
            });
        }

        let wait = {
            let guard = SHUTDOWN_DURATION.lock().unwrap();
            guard.unwrap_or(Duration::from_secs(10))
        };

        tokio::time::sleep(wait).await;
        println!("Graceful shutdown timeout reached, forcing exit.");
        std::process::exit(0);
    });
}
