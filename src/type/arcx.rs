/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::sync::{Arc, Mutex};

#[cfg(test)]
#[path = "test/arcx.rs"]
mod tests;

#[macro_export]
macro_rules! arcx {
    ($val:expr) => {
        $crate::ArcX::new($val)
    };
}

#[macro_export]
macro_rules! vmove {
    ($($v:ident),+ , $blk:block) => {
        {
            $(let $v = $v.clone();)+
            async move $blk
        }
    };
}

#[macro_export]
macro_rules! arcx_async {
    ($($v:ident),+, $blk:block) => {
        $crate::vmove!($($v),+, $blk)
    };
}

pub struct ArcX<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> ArcX<T> {
    pub fn new(val: T) -> Self {
        Self { inner: Arc::new(Mutex::new(val)) }
    }

    pub fn set(&self, val: T) {
        *self.inner.lock().unwrap() = val;
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.lock().unwrap().clone()
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }
}

impl<T> Clone for ArcX<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ArcX<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.inner.lock().unwrap();
        write!(f, "ArcX({:?})", *lock)
    }
}
