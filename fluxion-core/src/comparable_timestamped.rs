// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Trait combining comparable inner values with full timestamp bounds.
//!
//! This module defines [`ComparableTimestamped`], the primary trait for operators
//! that extract inner values and perform timestamp operations.

use crate::ComparableInner;
use std::fmt::Debug;

/// Combined trait for operators requiring comparable inner values with timestamp operations.
///
/// This supertrait bundles [`ComparableInner`] with explicit timestamp bounds,
/// providing the complete requirements for "Pattern A" operators.
///
/// # Trait Hierarchy Position
///
/// ```text
/// ComparableInner
///   └── ComparableTimestamped (this trait)
///         └── ComparableUnpinTimestamped
/// ```
///
/// # Why Use This?
///
/// While Rust still requires restating the associated type bounds (due to how trait
/// where clauses work), using `ComparableTimestamped` provides several benefits:
///
/// 1. **Clearer intent** - Signals that the operator needs both inner value and timestamp operations
/// 2. **Single source of truth** - All requirements defined in one place
/// 3. **Easier refactoring** - Change bounds once in the trait definition
/// 4. **Better documentation** - Self-documenting code that shows Pattern A usage
///
/// # Usage in Operators
///
/// ```rust
/// # use fluxion_core::{ComparableTimestamped, StreamItem};
/// # use futures::Stream;
/// # use std::fmt::Debug;
/// // With ComparableTimestamped (clearer intent)
/// pub trait MyOperatorExt<T>: Stream<Item = StreamItem<T>>
/// where
///     T: ComparableTimestamped,
///     T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
///     T::Timestamp: Clone + Debug + Ord + Send + Sync,
/// { }
/// ```
///
/// # Bounds Included
///
/// - `ComparableInner` - Base requirements for ordered items with comparable inner values
/// - `T::Inner: Clone + Debug + Ord + Send + Sync + 'static` - Inner value requirements
/// - `T::Timestamp: Clone + Debug + Ord + Send + Sync` - Timestamp requirements
///
/// # Pattern A Operators
///
/// Use this for operators that extract inner values and perform timestamp operations:
/// - `combine_latest` - Combines timestamped streams with inner value extraction
/// - `with_latest_from` - Joins streams using latest values and timestamps
///
/// # Note on Bounds Restatement
///
/// Due to Rust's trait system, associated type bounds must still be explicitly stated
/// when using this trait. The blanket implementation ensures type safety by requiring
/// all bounds to be satisfied before the trait can be implemented.
pub trait ComparableTimestamped: ComparableInner
where
    Self::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    Self::Timestamp: Clone + Debug + Ord + Send + Sync,
{
}

/// Blanket implementation for all types satisfying the requirements.
impl<T> ComparableTimestamped for T
where
    T: ComparableInner,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
{
}
