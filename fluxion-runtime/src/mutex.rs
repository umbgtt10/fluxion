// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::ops::{Deref, DerefMut};

pub trait MutexLike<T: ?Sized>: Clone {
    /// The guard type returned by `lock()`
    type Guard<'a>: Deref<Target = T> + DerefMut
    where
        Self: 'a,
        T: 'a;

    /// Create a new mutex wrapping the given value
    fn new(value: T) -> Self
    where
        T: Sized;

    /// Lock the mutex and return a guard
    fn lock(&self) -> Self::Guard<'_>;
}
