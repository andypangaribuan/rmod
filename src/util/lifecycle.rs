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
static WAIT_TX: LazyLock<broadcast::Sender<()>> = LazyLock::new(|| broadcast::channel(1).0);

pub fn subscribe() -> broadcast::Receiver<()> {
    SHUTDOWN_TX.subscribe()
}

/// Wait for the lifecycle shutdown process to complete.
pub async fn wait() {
    let mut rx = WAIT_TX.subscribe();
    let _ = rx.recv().await;
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

        let start_time = tokio::time::Instant::now();

        // 1. Notify all subscribers (Axum, Jobs, etc.)
        let _ = SHUTDOWN_TX.send(());

        let wait_duration = {
            let guard = SHUTDOWN_DURATION.lock().unwrap();
            (*guard).unwrap_or(Duration::from_secs(10))
        };

        // 2. Run the "before shutdown" callbacks FIRST.
        // We wait as long as they take to finish.
        let cbs = {
            let mut guard = CALLBACKS.lock().unwrap();
            std::mem::take(&mut *guard)
        };

        if !cbs.is_empty() {
            let mut handles = Vec::with_capacity(cbs.len());
            for cb in cbs {
                handles.push(tokio::spawn(cb()));
            }
            // Wait for all registered callbacks to finish completely
            let _ = futures_util::future::join_all(handles).await;
        }

        // 3. After callbacks are done, calculate if we still need to wait to reach SHUTDOWN_DURATION.
        let elapsed = start_time.elapsed();
        if elapsed < wait_duration {
            let remaining = wait_duration - elapsed;
            tokio::time::sleep(remaining).await;
        }

        // 4. Notify that the lifecycle task is done
        let _ = WAIT_TX.send(());

        // Give a tiny moment for main thread to exit naturally
        tokio::time::sleep(Duration::from_millis(50)).await;

        std::process::exit(0);
    });
}
