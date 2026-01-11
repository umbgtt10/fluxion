// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Timestamped;
use core::fmt::Debug;

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
///
/// For multi-threaded runtimes (tokio, smol, async-std), this requires `Send + Sync`.
/// For single-threaded runtimes (wasm, embassy), these bounds are not required.
// Multi-threaded runtimes (tokio/smol/async-std) - requires Send + Sync
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub trait Fluxion: Timestamped + Clone + Send + Sync + Unpin + 'static + Debug + Ord
where
    Self::Inner: Clone + Send + Sync + Unpin + 'static + Debug + Ord,
{
}

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T> Fluxion for T
where
    T: Timestamped + Clone + Send + Sync + Unpin + 'static + Debug + Ord,
    T::Inner: Clone + Send + Sync + Unpin + 'static + Debug + Ord,
{
}

// Single-threaded runtimes (wasm/embassy) - no Send + Sync required
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub trait Fluxion: Timestamped + Clone + Unpin + 'static + Debug + Ord
where
    Self::Inner: Clone + Unpin + 'static + Debug + Ord,
{
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<T> Fluxion for T
where
    T: Timestamped + Clone + Unpin + 'static + Debug + Ord,
    T::Inner: Clone + Unpin + 'static + Debug + Ord,
{
}
