/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::future::Future;
use std::pin::Pin;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::broadcast;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
type ShutdownCallback = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static>;

static SHUTDOWN_DURATION: LazyLock<Mutex<Option<Duration>>> = LazyLock::new(|| Mutex::new(None));
static CALLBACKS: LazyLock<Mutex<Vec<ShutdownCallback>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static STARTED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static SHUTDOWN_TX: LazyLock<broadcast::Sender<()>> = LazyLock::new(|| broadcast::channel(1).0);

pub fn subscribe() -> broadcast::Receiver<()> {
    SHUTDOWN_TX.subscribe()
}

pub fn before_graceful_shutdown<F, Fut>(callbacks: Vec<F>)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut guard = CALLBACKS.lock().unwrap();
    for cb in callbacks {
        guard.push(Box::new(move || Box::pin(cb()) as BoxFuture<'static, ()>));
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

        // Notify all subscribers (like Fuse server and background jobs)
        let _ = SHUTDOWN_TX.send(());

        let cbs = {
            let mut guard = CALLBACKS.lock().unwrap();
            std::mem::take(&mut *guard)
        };

        let mut handles = Vec::with_capacity(cbs.len());
        for cb in cbs {
            handles.push(tokio::spawn(cb()));
        }

        let wait = {
            let guard = SHUTDOWN_DURATION.lock().unwrap();
            guard.unwrap_or(Duration::from_secs(10))
        };

        if !handles.is_empty() {
            if tokio::time::timeout(wait, futures_util::future::join_all(handles)).await.is_err() {
                println!("graceful shutdown timeout reached, forcing exit");
            }
        }

        // Give a tiny grace period for the main thread and other tasks to realize we are shutting down
        tokio::time::sleep(Duration::from_millis(100)).await;

        std::process::exit(0);
    });
}
