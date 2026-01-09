// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{mutex::MutexLike, timer::Timer};
use core::cmp::Ord;
use core::default::Default;
use core::fmt::Debug;
use core::marker::Sized;
use core::marker::{Copy, Send, Sync};

pub trait Runtime: 'static {
    type Mutex<T: ?Sized>: MutexLike<T>;
    type Timer: Timer<Instant = Self::Instant> + Default;
    type Instant: Copy + Ord + Send + Sync + Debug;
}
