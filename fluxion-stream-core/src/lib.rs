// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod combine_latest;
pub mod combine_with_previous;
pub mod distinct_until_changed;
pub mod distinct_until_changed_by;
pub mod emit_when;
pub mod filter_ordered;
pub mod map_ordered;
pub mod merge_with;
pub mod on_error;
pub mod ordered_merge;
pub mod partition;
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

pub use combine_latest::{combine_latest_impl, CombinedState};
pub use combine_with_previous::{combine_with_previous_impl, WithPrevious};
pub use emit_when::emit_when_impl;
pub use merge_with::MergedStream;
pub use ordered_merge::ordered_merge_impl;
pub use partition::{partition_impl, PartitionedStream};
pub use share::{FluxionShared, SharedBoxStream};
pub use take_latest_when::{take_latest_when_impl, take_latest_when_impl_single};
pub use take_while_with::take_while_with_impl;
pub use with_latest_from::with_latest_from_impl;
