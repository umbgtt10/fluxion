// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Prelude module with all commonly used traits and types (single-threaded runtime version)

pub use crate::combine_latest::{CombineLatestExt, CombinedState};
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
// Note: partition and share excluded - single-threaded runtimes can't spawn background tasks
pub use crate::sample_ratio::SampleRatioExt;
pub use crate::scan_ordered::ScanOrderedExt;
pub use crate::skip_items::SkipItemsExt;
pub use crate::start_with::StartWithExt;
pub use crate::take_items::TakeItemsExt;
pub use crate::take_latest_when::TakeLatestWhenExt;
pub use crate::take_while_with::TakeWhileExt;
pub use crate::tap::TapExt;
pub use crate::window_by_count::WindowByCountExt;
pub use crate::with_latest_from::WithLatestFromExt;
pub use fluxion_stream_core::combine_with_previous::WithPrevious;
