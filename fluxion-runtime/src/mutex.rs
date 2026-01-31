// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::clone::Clone;
use core::marker::Sized;
use core::ops::{Deref, DerefMut};

pub trait MutexLike<T: ?Sized>: Clone {
    type Guard<'a>: Deref<Target = T> + DerefMut
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self
    where
        T: Sized;

    fn lock(&self) -> Self::Guard<'_>;
}
