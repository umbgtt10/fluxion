// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Pairing operator that combines each item with its predecessor.
//!
//! The [`combine_with_previous`](CombineWithPreviousExt::combine_with_previous) operator
//! emits `WithPrevious` items, containing the current value and the previous value (if any).
//!
//! # Behavior
//!
//! - First element: `WithPrevious { previous: None, current: first_value }`
//! - Subsequent elements: `WithPrevious { previous: Some(prev), current: curr }`
//! - Maintains state to track the previous value
//! - Preserves temporal ordering from the source stream
//!
//! # Returns
//!
//! A stream of `WithPrevious<T>` where each item contains the current
//! and previous values.
//!
//! # Errors
//!
//! This operator may produce `StreamItem::Error` in the following cases:
//!
//! - **Lock Errors**: When acquiring the previous value buffer lock fails (e.g., due to lock poisoning).
//!   These are transient errors - the stream continues processing and may succeed on subsequent items.
//!
//! Lock errors are typically non-fatal and indicate temporary contention. The operator will continue
//! processing subsequent items. See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for patterns
//! on handling these errors in your application.
//!
//! # See Also
//!
//! - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Combines multiple streams
//! - Useful for change detection and delta calculations
//!
//! # Examples
//!
//! ```rust
//! use fluxion_stream::CombineWithPreviousExt;
//! use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
//! use fluxion_core::Timestamped as TimestampedTrait;
//!
//! # async fn example() {
//! // Create channel
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! // Combine with previous
//! let mut paired = stream.combine_with_previous();
//!
//! // Send values
//! tx.unbounded_send((1, 1).into()).unwrap();
//! tx.unbounded_send((2, 2).into()).unwrap();
//!
//! // Assert - first has no previous
//! let first = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
//! assert_eq!(first.previous, None);
//! assert_eq!(first.current.value, 1);
//!
//! // Assert - second has previous
//! let second = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
//! assert_eq!(second.previous.as_ref().unwrap().value, 1);
//! assert_eq!(second.current.value, 2);
//! # }
//! ```
//!
//! # Use Cases
//!
//! - Change detection (comparing consecutive values)
//! - Delta calculation (computing differences)
//! - State transitions (analyzing previous ? current)
//! - Duplicate filtering (skip if same as previous)

#[macro_use]
mod implementation;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
mod multi_threaded;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub use multi_threaded::CombineWithPreviousExt;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
mod single_threaded;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub use single_threaded::CombineWithPreviousExt;
