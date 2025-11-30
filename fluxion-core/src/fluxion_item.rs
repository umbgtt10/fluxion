// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Timestamped;
use std::fmt::Debug;

/// The standard trait for items flowing through Fluxion streams.
///
/// This trait aggregates all the necessary bounds for a type to be used
/// with Fluxion operators. It ensures the type is:
/// - Timestamped (for ordering)
/// - Thread-safe (Send + Sync)
/// - Movable (Unpin)
/// - Debuggable
/// - Orderable
/// - 'static (owned data)
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for any type that satisfies the bounds.
pub trait Fluxion: Timestamped + Clone + Send + Sync + Unpin + 'static + Debug + Ord
where
    Self::Inner: Clone + Send + Sync + Unpin + 'static + Debug + Ord,
{
}

impl<T> Fluxion for T
where
    T: Timestamped + Clone + Send + Sync + Unpin + 'static + Debug + Ord,
    T::Inner: Clone + Send + Sync + Unpin + 'static + Debug + Ord,
{
}
