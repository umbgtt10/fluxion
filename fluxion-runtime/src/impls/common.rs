// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-wasm",
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
use std::sync::Arc;

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
impl<T: ?Sized> MutexLike<T> for Arc<parking_lot::Mutex<T>> {
    type Guard<'a>
        = parking_lot::MutexGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self
    where
        T: Sized,
    {
        Arc::new(parking_lot::Mutex::new(value))
    }

    fn lock(&self) -> Self::Guard<'_> {
        self.as_ref().lock()
    }
}
