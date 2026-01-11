// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
extern crate alloc;

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
use alloc::sync::Arc;

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
use parking_lot::{Mutex, MutexGuard};

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
use crate::mutex::MutexLike;

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T: ?Sized> MutexLike<T> for Arc<Mutex<T>> {
    type Guard<'a>
        = MutexGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self
    where
        T: Sized,
    {
        Arc::new(Mutex::new(value))
    }

    fn lock(&self) -> Self::Guard<'_> {
        self.as_ref().lock()
    }
}
