// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Trait for operators requiring comparable, unpin inner values with timestamps.
//!
//! This module defines [`ComparableUnpinTimestamped`], used by operators that need
//! unpinned comparable values with full timestamp operations.

use crate::ComparableTimestamped;
use std::fmt::Debug;

/// Trait for operators requiring comparable, unpin inner values with timestamped operations.
///
/// This extends [`ComparableTimestamped`] by adding the `Unpin` requirement on associated types.
/// The `Unpin` bound is necessary for operators that need to move pinned futures or streams
/// containing these values.
///
/// # Trait Hierarchy Position
///
/// ```text
/// ComparableInner
///   └── ComparableTimestamped
///         └── ComparableUnpinTimestamped (this trait)
/// ```
///
/// # When to Use
///
/// Use this instead of [`ComparableTimestamped`] when your operator:
/// - Stores values across `.await` points in non-`Pin` contexts
/// - Boxes streams containing these values without `Pin`
/// - Requires inner values that can be safely moved in memory
///
/// # Usage
///
/// ```rust
/// # use fluxion_core::{ComparableUnpinTimestamped, StreamItem};
/// # use futures::Stream;
/// # use std::fmt::Debug;
/// pub trait MyUnpinOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: ComparableUnpinTimestamped,
///     T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
///     T::Timestamp: Clone + Debug + Ord + Send + Sync,
/// { }
/// ```
///
/// # Bounds Included
///
/// Everything from [`ComparableTimestamped`] plus:
/// - `T::Inner: Unpin` - Inner values can be moved safely
///
/// # Operators Using This
///
/// - `take_while_with` - Gates stream with unpinned filter values
/// - Other operators needing unpinned value manipulation
pub trait ComparableUnpinTimestamped: ComparableTimestamped
where
    Self::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    Self::Timestamp: Clone + Debug + Ord + Send + Sync,
{
}

/// Blanket implementation for all types satisfying the Unpin requirements.
impl<T> ComparableUnpinTimestamped for T
where
    T: ComparableTimestamped,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
{
}
