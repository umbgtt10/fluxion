// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::ordered::Ordered;
use std::{
    cmp::Ordering,
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};

static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A wrapper that adds automatic sequencing to any value for temporal ordering.
///
/// Uses a monotonically increasing sequence counter (logical timestamp) to establish
/// a total ordering of events. The sequence is assigned when the value is created.
#[derive(Debug, Clone)]
pub struct Sequenced<T> {
    pub value: T,
    sequence: u64,
}

impl<T: PartialEq> PartialEq for Sequenced<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.sequence == other.sequence
    }
}

impl<T: Eq> Eq for Sequenced<T> {}

impl<T> Sequenced<T> {
    /// Creates a new sequenced value with an automatically assigned sequence number.
    pub fn new(value: T) -> Self {
        Self {
            value,
            sequence: GLOBAL_SEQUENCE.fetch_add(1, SeqCst),
        }
    }

    pub const fn with_sequence(value: T, sequence: u64) -> Self {
        Self { value, sequence }
    }

    /// Gets the inner value, consuming the wrapper.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Gets a reference to the inner value.
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// Gets a mutable reference to the inner value.
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Gets the sequence number.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl<T: PartialEq> PartialOrd for Sequenced<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sequence.partial_cmp(&other.sequence)
    }
}

impl<T: Eq> Ord for Sequenced<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence.cmp(&other.sequence)
    }
}

impl<T> Deref for Sequenced<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Sequenced<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Display> fmt::Display for Sequenced<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: Clone> Ordered for Sequenced<T> {
    type Inner = T;

    fn order(&self) -> u64 {
        self.sequence
    }

    fn get(&self) -> &T {
        &self.value
    }

    fn with_order(value: T, order: u64) -> Self {
        Self::with_sequence(value, order)
    }

    fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Ord> fluxion_core::CompareByInner for Sequenced<T> {
    fn cmp_inner(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> From<(T, u64)> for Sequenced<T> {
    fn from((value, order): (T, u64)) -> Self {
        Self::with_sequence(value, order)
    }
}

// Special conversion for Empty streams - this will never actually be called
// since Empty streams never yield items, but it's needed for type checking
impl<T> From<()> for Sequenced<T> {
    fn from((): ()) -> Self {
        unreachable!("Empty streams never yield items, so this conversion should never be called")
    }
}
