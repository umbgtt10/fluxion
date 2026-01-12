// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Prelude module re-exporting all commonly used traits and types.
//!
//! Import this module for convenient access to all Fluxion stream operators:
//!
//! ```ignore
//! use fluxion_stream::prelude::*;
//!
//! // Now all extension traits are available
//! let processed = stream
//!     .map_ordered(...)
//!     .filter_ordered(...)
//!     .combine_with_previous()
//!     .share();
//! ```
//!
//! # Contents
//!
//! ## Extension Traits (Operators)
//!
//! - [`CombineLatestExt`] - Combine latest values from multiple streams
//! - [`CombineWithPreviousExt`] - Pair each value with its predecessor
//! - [`DistinctUntilChangedExt`] - Suppress consecutive duplicates
//! - [`DistinctUntilChangedByExt`] - Suppress duplicates by custom comparison
//! - [`EmitWhenExt`] - Gate emissions based on condition
//! - [`FilterOrderedExt`] - Filter items preserving temporal order
//! - [`MapOrderedExt`] - Transform items preserving temporal order
//! - [`OnErrorExt`] - Handle stream errors
//! - [`OrderedStreamExt`] - Merge streams with temporal ordering
//! - [`ScanOrderedExt`] - Stateful accumulation
//! #[cfg(any(feature = "runtime-tokio", feature = "runtime-smol", feature = "runtime-async-std", target_arch = "wasm32"))]
//! - [`ShareExt`] - Convert stream to multi-subscriber source
//! - [`SkipItemsExt`] - Skip first n items
//! - [`StartWithExt`] - Prepend initial values
//! - [`TakeItemsExt`] - Take first n items
//! - [`TakeLatestWhenExt`] - Sample on trigger events
//! - [`TakeWhileExt`] - Take while condition holds
//! - [`TapExt`] - Side-effect observation for debugging
//! - [`WindowByCountExt`] - Batch items into fixed-size windows
//! - [`WithLatestFromExt`] - Combine with latest from secondary streams
//! - [`IntoFluxionStream`] - Convert receivers to streams
//!
//! ## Types
//!
//! - [`CombinedState`] - Combined state from multiple streams
//! - [`WithPrevious`] - Pair of current and previous values
//! #[cfg(any(feature = "runtime-tokio", feature = "runtime-smol", feature = "runtime-async-std", target_arch = "wasm32"))]
//! - [`FluxionShared`] - Multi-subscriber subscription factory
//! - [`MergedStream`] - Merged stream type

pub use crate::combine_latest::CombineLatestExt;
pub use crate::combine_with_previous::CombineWithPreviousExt;
pub use crate::distinct_until_changed::DistinctUntilChangedExt;
pub use crate::distinct_until_changed_by::DistinctUntilChangedByExt;
pub use crate::emit_when::EmitWhenExt;
pub use crate::filter_ordered::FilterOrderedExt;
pub use crate::into_fluxion_stream::IntoFluxionStream;
pub use crate::map_ordered::MapOrderedExt;
pub use crate::merge_with::MergedStream;
pub use crate::on_error::OnErrorExt;
pub use crate::ordered_merge::OrderedStreamExt;
#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-smol",
    feature = "runtime-async-std",
    target_arch = "wasm32"
))]
pub use crate::partition::{PartitionExt, PartitionedStream};
pub use crate::sample_ratio::SampleRatioExt;
pub use crate::scan_ordered::ScanOrderedExt;
#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-smol",
    feature = "runtime-async-std",
    target_arch = "wasm32"
))]
pub use crate::share::{FluxionShared, ShareExt};
pub use crate::skip_items::SkipItemsExt;
pub use crate::start_with::StartWithExt;
pub use crate::take_items::TakeItemsExt;
pub use crate::take_latest_when::TakeLatestWhenExt;
pub use crate::take_while_with::TakeWhileExt;
pub use crate::tap::TapExt;
pub use crate::types::{CombinedState, WithPrevious};
pub use crate::window_by_count::WindowByCountExt;
pub use crate::with_latest_from::WithLatestFromExt;
