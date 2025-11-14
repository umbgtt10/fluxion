// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// A trait for types that have an intrinsic ordering for stream merging.
///
/// This trait allows stream operators like `combine_latest` and `ordered_merge`
/// to work with any type that can provide an ordering value and wraps an inner value.
///
/// # Type Parameters
/// * `Inner` - The type of the value wrapped by this ordered type
///
/// # Examples
///
/// ```
/// use fluxion_core::Ordered;
///
/// #[derive(Clone, Debug)]
/// struct Timestamped<T> {
///     value: T,
///     timestamp: u64,
/// }
///
/// impl<T: Clone> Ordered for Timestamped<T> {
///     type Inner = T;
///
///     fn order(&self) -> u64 {
///         self.timestamp
///     }
///
///     fn get(&self) -> &T {
///         &self.value
///     }
///
///     fn with_order(value: T, order: u64) -> Self {
///         Timestamped { value, timestamp: order }
///     }
/// }
/// ```
pub trait Ordered: Clone {
    /// The type of the inner value wrapped by this ordered type
    type Inner: Clone;

    /// Returns the ordering value for this item.
    /// Stream operators use this to determine the order of items.
    fn order(&self) -> u64;

    /// Gets a reference to the inner value.
    fn get(&self) -> &Self::Inner;

    /// Creates a new instance wrapping the given value with the specified order.
    fn with_order(value: Self::Inner, order: u64) -> Self;

    /// Gets the inner value, consuming the wrapper.
    fn into_inner(self) -> Self::Inner {
        // Default implementation using clone, can be overridden for efficiency
        self.get().clone()
    }

    /// Creates a new instance with a different inner type.
    /// This is used by operators like `combine_latest` that transform the inner value.
    fn map_inner<F, U>(self, f: F, order: u64) -> impl Ordered<Inner = U>
    where
        F: FnOnce(Self::Inner) -> U,
        U: Clone,
        Self: Sized,
    {
        OrderedWrapper {
            value: f(self.into_inner()),
            order,
        }
    }
}

/// A simple wrapper type for Ordered values with transformed inner types
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderedWrapper<T> {
    value: T,
    order: u64,
}

impl<T: Clone> Ordered for OrderedWrapper<T> {
    type Inner = T;

    fn order(&self) -> u64 {
        self.order
    }

    fn get(&self) -> &T {
        &self.value
    }

    fn with_order(value: T, order: u64) -> Self {
        Self { value, order }
    }

    fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Clone + PartialOrd> PartialOrd for OrderedWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl<T: Clone + Ord> Ord for OrderedWrapper<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}
