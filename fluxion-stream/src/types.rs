// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Common types and type aliases used throughout the fluxion-stream crate.
//!
//! This module centralizes shared types to reduce duplication and improve maintainability.

use fluxion_core::Ordered;
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

    /// Returns a tuple of references to both values if previous exists.
    pub fn both(&self) -> Option<(&T, &T)> {
        self.previous.as_ref().map(|prev| (prev, &self.current))
    }
}

impl<T: Ordered> Ordered for WithPrevious<T> {
    type Inner = T::Inner;

    fn order(&self) -> u64 {
        self.current.order()
    }

    fn get(&self) -> &Self::Inner {
        self.current.get()
    }

    fn with_order(value: Self::Inner, order: u64) -> Self {
        Self {
            previous: None,
            current: T::with_order(value, order),
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
/// let state = CombinedState::new(vec![1, 2, 3]);
/// assert_eq!(state.get_state().len(), 3);
/// assert_eq!(state.get_state()[0], 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombinedState<V>
where
    V: Clone + Debug + Ord,
{
    state: Vec<V>,
}

impl<V> CombinedState<V>
where
    V: Clone + Debug + Ord,
{
    /// Creates a new CombinedState with the given vector of values.
    pub fn new(state: Vec<V>) -> Self {
        Self { state }
    }

    /// Returns a reference to the internal state vector.
    pub fn get_state(&self) -> &Vec<V> {
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

impl<V> Ordered for CombinedState<V>
where
    V: Clone + Debug + Ord,
{
    type Inner = Self;

    fn order(&self) -> u64 {
        // CombinedState doesn't have its own order - it's always wrapped by an Ordered type
        // This should never be called directly
        0
    }

    fn get(&self) -> &Self::Inner {
        self
    }

    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}


/// Type alias for the common trait bounds used for ordered stream items.
///
/// This trait requires that types are:
/// - `Ordered`: Have temporal ordering
/// - `Clone`: Can be duplicated
/// - `Debug`: Can be formatted for debugging
/// - `Ord`: Have total ordering
/// - `Send + Sync`: Can be safely transferred between threads
/// - `Unpin`: Can be moved after being pinned
/// - `'static`: Contains no non-static references
///
/// # Usage
///
/// Instead of writing:
/// ```
/// # use fluxion_stream::OrderedStreamItem;
/// # use fluxion_core::Ordered;
/// # use std::fmt::Debug;
/// fn process_stream<T>()
/// where
///     T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static
/// # {}
/// ```
///
/// You can write:
/// ```
/// # use fluxion_stream::OrderedStreamItem;
/// fn process_stream<T>()
/// where
///     T: OrderedStreamItem
/// # {}
/// ```
pub trait OrderedStreamItem: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static {}

// Blanket implementation for all types that satisfy the bounds
impl<T> OrderedStreamItem for T where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static
{
}

/// Type alias for the common trait bounds used for the inner values of ordered stream items.
///
/// This trait requires that inner types are:
/// - `Clone`: Can be duplicated
/// - `Debug`: Can be formatted for debugging
/// - `Ord`: Have total ordering
/// - `Send + Sync`: Can be safely transferred between threads
/// - `'static`: Contains no non-static references
///
/// # Usage
///
/// Instead of writing:
/// ```
/// # use std::fmt::Debug;
/// fn process_inner<T>()
/// where
///     T: Clone + Debug + Ord + Send + Sync + 'static
/// # {}
/// ```
///
/// You can write:
/// ```
/// # use fluxion_stream::OrderedInner;
/// fn process_inner<T>()
/// where
///     T: OrderedInner
/// # {}
/// ```
pub trait OrderedInner: Clone + Debug + Ord + Send + Sync + 'static {}

// Blanket implementation for all types that satisfy the bounds
impl<T> OrderedInner for T where T: Clone + Debug + Ord + Send + Sync + 'static {}

/// Type alias for the common trait bounds used for ordered items that need to be
/// unwrapped (without Unpin requirement).
///
/// Used in scenarios where the inner value needs to implement Ordered but doesn't
/// need to be Unpin.
pub trait OrderedInnerUnwrapped: Clone + Debug + Ord + Send + Sync + Unpin + 'static {}

// Blanket implementation
impl<T> OrderedInnerUnwrapped for T where T: Clone + Debug + Ord + Send + Sync + Unpin + 'static {}
