/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test_defer_macro() {
    let executed = Arc::new(AtomicBool::new(false));

    {
        let executed_clone = executed.clone();
        crate::defer! {
            executed_clone.store(true, Ordering::SeqCst);
        };

        // Should not be executed yet
        assert!(!executed.load(Ordering::SeqCst));
    }

    // Should be executed now that the scope has ended
    assert!(executed.load(Ordering::SeqCst));
}

#[test]
fn test_multiple_defers() {
    let order = Arc::new(std::sync::Mutex::new(Vec::new()));

    {
        let order_clone1 = order.clone();
        crate::defer! {
            order_clone1.lock().unwrap().push(1);
        };

        let order_clone2 = order.clone();
        crate::defer! {
            order_clone2.lock().unwrap().push(2);
        };
    }

    // Defer should execute in LIFO order (Last-In, First-Out)
    let final_order = order.lock().unwrap().clone();
    assert_eq!(final_order, vec![2, 1]);
}
