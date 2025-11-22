// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Trait for operators requiring comparable, thread-safe inner values without 'static.
//!
//! This module defines [`ComparableSync`], used by operators that need thread-safe
//! inner values but don't require the `'static` lifetime constraint.

use crate::OrderedFluxionItem;
use std::fmt::Debug;

/// Requirements for operators needing thread-safe inner values without 'static lifetime.
///
/// This trait is for operators that extract and compare inner values across threads
/// but don't need to store them beyond the operator's lifetime.
///
/// # Trait Hierarchy Position
///
/// ```text
/// OrderedFluxionItem
///   └── ComparableSync (this trait)
/// ```
///
/// Parallel to `ComparableInner` but without the `'static` requirement.
///
/// # When to Use
///
/// Use this for operators that:
/// - Extract and compare inner values
/// - Pass values between threads
/// - Don't store values in `'static` contexts
/// - Work with borrowed or scoped data
///
/// # Bounds Included
///
/// - `OrderedFluxionItem` - Base timestamped, ordered type
/// - `T::Inner: Clone + Debug + Ord + Send + Sync` - Thread-safe comparable inner values
/// - **Note**: No `'static` requirement on `T::Inner`
///
/// # Operators Using This
///
/// - `emit_when` - Gates emissions without storing values permanently
/// - `take_latest_when` - Samples values without long-term storage
///
/// # Example
///
/// ```rust
/// # use fluxion_core::{ComparableSync, StreamItem};
/// # use futures::Stream;
/// # use std::fmt::Debug;
/// pub trait MyGatingOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: ComparableSync,
///     T::Inner: Clone + Debug + Ord + Send + Sync,
/// {
///     // Operator that gates but doesn't permanently store
/// }
/// ```
pub trait ComparableSync: OrderedFluxionItem
where
    Self::Inner: Clone + Debug + Ord + Send + Sync,
{
}

/// Blanket implementation for all types satisfying the requirements.
impl<T> ComparableSync for T
where
    T: OrderedFluxionItem,
    T::Inner: Clone + Debug + Ord + Send + Sync,
{
}
