// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use chrono::{DateTime, Utc};
use fluxion_core::Timestamped;
use std::{
    cmp::Ordering,
    fmt,
    ops::{Deref, DerefMut},
};

/// A wrapper that adds automatic timestamping to any value for temporal ordering.
///
/// Uses chrono timestamps to establish a total ordering of events.
/// The timestamp is assigned when the value is created using the current UTC time.
#[derive(Debug, Clone)]
pub struct ChronoTimestamped<T> {
    pub value: T,
    timestamp: DateTime<Utc>,
}

impl<T: PartialEq> PartialEq for ChronoTimestamped<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.timestamp == other.timestamp
    }
}

impl<T: Eq> Eq for ChronoTimestamped<T> {}

impl<T> ChronoTimestamped<T> {
    /// Creates a new timestamped value with the current UTC time.
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
        }
    }

    /// Creates a timestamped value with a specific timestamp.
    pub fn with_timestamp_value(value: T, timestamp: DateTime<Utc>) -> Self {
        Self { value, timestamp }
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
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Gets the timestamp.
    pub const fn get_timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

impl<T: PartialEq> PartialOrd for ChronoTimestamped<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl<T: Eq> Ord for ChronoTimestamped<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<T> Deref for ChronoTimestamped<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for ChronoTimestamped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Display> fmt::Display for ChronoTimestamped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: Clone> Timestamped for ChronoTimestamped<T> {
    type Inner = T;
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        // Convert DateTime to nanoseconds since Unix epoch
        self.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64
    }

    fn with_timestamp(value: T, timestamp: u64) -> Self {
        // Convert u64 nanoseconds back to DateTime
        let secs = (timestamp / 1_000_000_000) as i64;
        let nsecs = (timestamp % 1_000_000_000) as u32;
        let dt = DateTime::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now);
        Self::with_timestamp_value(value, dt)
    }

    fn with_fresh_timestamp(value: T) -> Self {
        Self::new(value)
    }

    fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Ord> fluxion_core::CompareByInner for ChronoTimestamped<T> {
    fn cmp_inner(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Clone> From<(T, u64)> for ChronoTimestamped<T> {
    fn from((value, timestamp): (T, u64)) -> Self {
        <Self as Timestamped>::with_timestamp(value, timestamp)
    }
}

// Special conversion for Empty streams - this will never actually be called
// since Empty streams never yield items, but it's needed for type checking
impl<T> From<()> for ChronoTimestamped<T> {
    fn from((): ()) -> Self {
        unreachable!("Empty streams never yield items, so this conversion should never be called")
    }
}
