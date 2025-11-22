// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Base trait for operators requiring comparable inner values.
//!
//! This module defines [`ComparableInner`], the foundation of the trait hierarchy
//! for operators that extract and manipulate inner values.

use crate::OrderedFluxionItem;
use std::fmt::Debug;

/// Base requirements for operators that extract and manipulate inner values.
///
/// This trait extends [`OrderedFluxionItem`] with bounds on the associated `Inner` type,
/// establishing the foundation for all "comparable inner" operator patterns.
///
/// # Trait Hierarchy
///
/// ```text
/// ComparableInner (base)
///   ├── ComparableSync (adds thread safety without 'static)
///   ├── ComparableTimestamped (adds 'static + timestamp bounds)
///   └── ComparableUnpinTimestamped (adds Unpin)
/// ```
///
/// # When to Use
///
/// Use this as the base bound when operators:
/// - Extract inner values via `into_inner()`
/// - Need to compare extracted values
/// - Need to debug extracted values
/// - Send inner values across threads
///
/// # Bounds Included
///
/// - `OrderedFluxionItem` - Base timestamped, ordered type
/// - `T::Inner: Clone` - Can clone extracted inner values
/// - `T::Inner: Debug` - Can debug extracted inner values
/// - `T::Inner: Ord` - Can compare extracted inner values
/// - `T::Inner: Send + Sync` - Can send inner values across threads
/// - `T::Inner: 'static` - Inner values have no lifetime constraints
///
/// # Example
///
/// ```rust
/// # use fluxion_core::{ComparableInner, StreamItem};
/// # use futures::Stream;
/// # use std::fmt::Debug;
/// pub trait MyOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: ComparableInner,
///     T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
/// {
///     // Operator that extracts and compares inner values
/// }
/// ```
pub trait ComparableInner: OrderedFluxionItem
where
    Self::Inner: Clone + Debug + Ord + Send + Sync + 'static,
{
}

/// Blanket implementation for all types satisfying the requirements.
impl<T> ComparableInner for T
where
    T: OrderedFluxionItem,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
{
}
