// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestWrapper<T> {
    timestamp: u64,
    value: T,
}

impl<T> TestWrapper<T> {
    pub fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> HasTimestamp for TestWrapper<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T> Timestamped for TestWrapper<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.value
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { value, timestamp }
    }
}
