// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::OrderedFluxionItem;

/// Trait for ordered items with unpinned, comparable inner values.
///
/// This trait extends [`OrderedFluxionItem`] with the requirement that the inner value
/// implements `Unpin`, allowing the value to be safely moved in memory. This is necessary
/// for operators that need to store or move the inner values, such as `map_ordered`.
///
/// # Trait Hierarchy
///
/// ```text
/// ComparableUnpin
///   requires: T: OrderedFluxionItem
///   requires: T::Inner: Clone + Debug + Ord + Send + Sync + Unpin
/// ```
///
/// This is a lighter-weight alternative to [`ComparableUnpinTimestamped`](super::ComparableUnpinTimestamped)
/// for operators that don't require timestamp bounds.
///
/// # When to Use
///
/// Use `ComparableUnpin` for operators that:
/// - Need to move or store inner values
/// - Don't require timestamp operations
/// - Work with ordered items
///
/// Examples: `map_ordered`, `filter_ordered`
///
/// # Rust Trait Bounds Limitation
///
/// **Important**: Even though this trait contains all the necessary bounds in its definition,
/// Rust does not propagate where clause bounds from trait definitions to usage sites.
/// You must **explicitly restate** all bounds when using this trait:
///
/// ```rust
/// # use fluxion_core::ComparableUnpin;
/// # use std::fmt::Debug;
/// fn my_operator<T>(_item: T)
/// where
///     T: ComparableUnpin,
///     T::Inner: Clone + Debug + Ord + Send + Sync + Unpin,  // Must be explicit!
/// {
///     // operator implementation
/// }
/// ```
///
/// The trait serves as semantic documentation and a constraint, but does not eliminate
/// the need for explicit bounds.
pub trait ComparableUnpin: OrderedFluxionItem
where
    Self::Inner: Clone + std::fmt::Debug + Ord + Send + Sync + Unpin,
{
}

// Blanket implementation for all types that satisfy the bounds
impl<T> ComparableUnpin for T
where
    T: OrderedFluxionItem,
    T::Inner: Clone + std::fmt::Debug + Ord + Send + Sync + Unpin,
{
}
