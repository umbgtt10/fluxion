// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Common types and type aliases used throughout the fluxion-stream crate.
//!
//! This module centralizes shared types to reduce duplication and improve maintainability.

use fluxion_core::{HasTimestamp, Timestamped};
use std::fmt::Debug;

/// Represents a value paired with its previous value in the stream.
///
/// Used by [`CombineWithPreviousExt`](crate::CombineWithPreviousExt) to provide
/// both current and previous values.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithPrevious<T> {
    /// The previous value in the stream, if any
    pub previous: Option<T>,
    /// The current value in the stream
    pub current: T,
}

impl<T> WithPrevious<T> {
    /// Creates a new WithPrevious with the given previous and current values.
    pub fn new(previous: Option<T>, current: T) -> Self {
        Self { previous, current }
    }

    /// Returns true if there is a previous value.
    pub fn has_previous(&self) -> bool {
        self.previous.is_some()
    }

    /// Returns a tuple of references to (previous, current) if previous exists.
    pub fn as_pair(&self) -> Option<(&T, &T)> {
        self.previous.as_ref().map(|prev| (prev, &self.current))
    }
}

impl<T: Timestamped> HasTimestamp for WithPrevious<T> {
    type Inner = T::Inner;
    type Timestamp = T::Timestamp;

    fn timestamp(&self) -> Self::Timestamp {
        self.current.timestamp()
    }
}

impl<T: Timestamped> Timestamped for WithPrevious<T> {
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            previous: None,
            current: T::with_timestamp(value, timestamp),
        }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self {
            previous: None,
            current: T::with_fresh_timestamp(value),
        }
    }

    fn into_inner(self) -> Self::Inner {
        self.current.into_inner()
    }
}

/// State container holding the latest values from multiple combined streams.
///
/// Used by operators that combine multiple streams such as [`combine_latest`](crate::CombineLatestExt::combine_latest),
/// [`with_latest_from`](crate::WithLatestFromExt::with_latest_from), and
/// [`emit_when`](crate::EmitWhenExt::emit_when).
///
/// # Examples
///
/// ```
/// use fluxion_stream::CombinedState;
///
/// let state = CombinedState::new(vec![1, 2, 3], 0);
/// assert_eq!(state.values().len(), 3);
/// assert_eq!(state.values()[0], 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombinedState<V, TS = u64>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord,
{
    state: Vec<V>,
    timestamp: TS,
}

impl<V, TS> CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord,
{
    /// Creates a new CombinedState with the given vector of values and timestamp.
    pub fn new(state: Vec<V>, timestamp: TS) -> Self {
        Self { state, timestamp }
    }

    /// Returns a reference to the internal values vector.
    pub fn values(&self) -> &Vec<V> {
        &self.state
    }

    /// Returns the number of streams in the combined state.
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Returns true if there are no streams in the combined state.
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }
}

impl<V, TS> HasTimestamp for CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord + Copy + Send + Sync,
{
    type Inner = Self;
    type Timestamp = TS;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<V, TS> Timestamped for CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord + Copy + Send + Sync,
{
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            state: value.state,
            timestamp,
        }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        // For now, recycle the timestamp from the value itself
        // Later we can discuss whether to create a fresh one or use one from aggregated events
        value
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
