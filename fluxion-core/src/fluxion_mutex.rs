// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Synchronization primitives that switch between `parking_lot` (std) and `spin` (no_std).

#[cfg(feature = "std")]
pub use parking_lot::Mutex;

#[cfg(not(feature = "std"))]
pub use spin::Mutex;
