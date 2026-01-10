// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Fluxion stream operators for multi-threaded runtimes
//!
//! This crate provides stream operator traits optimized for multi-threaded runtimes
//! (Tokio, smol, async-std). All operators require Send + Sync bounds.

pub mod combine_latest;
pub mod combine_with_previous;
pub mod distinct_until_changed;
pub mod distinct_until_changed_by;
pub mod emit_when;
pub mod filter_ordered;
pub mod into_fluxion_stream;
pub mod map_ordered;
pub mod merge_with;
pub mod on_error;
pub mod ordered_merge;
pub mod partition;
pub mod prelude;
pub mod sample_ratio;
pub mod scan_ordered;
pub mod share;
pub mod skip_items;
pub mod start_with;
pub mod take_items;
pub mod take_latest_when;
pub mod take_while_with;
pub mod tap;
pub mod window_by_count;
pub mod with_latest_from;

pub use combine_latest::{CombineLatestExt, CombinedState};
pub use combine_with_previous::CombineWithPreviousExt;
pub use distinct_until_changed::DistinctUntilChangedExt;
pub use distinct_until_changed_by::DistinctUntilChangedByExt;
pub use emit_when::EmitWhenExt;
pub use filter_ordered::FilterOrderedExt;
pub use fluxion_stream_core::combine_with_previous::WithPrevious;
pub use into_fluxion_stream::IntoFluxionStream;
pub use map_ordered::MapOrderedExt;
pub use merge_with::MergedStream;
pub use on_error::OnErrorExt;
pub use ordered_merge::OrderedStreamExt;
pub use partition::{PartitionExt, PartitionedStream};
pub use sample_ratio::SampleRatioExt;
pub use scan_ordered::ScanOrderedExt;
pub use share::{FluxionShared, ShareExt, SharedBoxStream};
pub use skip_items::SkipItemsExt;
pub use start_with::StartWithExt;
pub use take_items::TakeItemsExt;
pub use take_latest_when::TakeLatestWhenExt;
pub use take_while_with::TakeWhileExt;
pub use tap::TapExt;
pub use window_by_count::WindowByCountExt;
pub use with_latest_from::WithLatestFromExt;
