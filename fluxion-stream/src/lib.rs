// Copyright 2025 Umberto Gotti
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions)]
#[macro_use]
mod logging;
pub mod combine_latest;
pub mod combine_with_previous;
pub mod fluxion_stream;
pub mod ordered_merge;
pub mod take_latest_when;
pub mod take_while_with;
pub mod util;
pub mod with_latest_from;

// Re-export commonly used types
pub use combine_latest::{CombineLatestExt, CombinedState};
pub use combine_with_previous::CombineWithPreviousExt;
pub use fluxion_core::{CompareByInner, Ordered, OrderedWrapper};
pub use fluxion_stream::FluxionStream;
pub use ordered_merge::OrderedStreamExt;
pub use take_latest_when::TakeLatestWhenExt;
pub use take_while_with::TakeWhileExt;
pub use with_latest_from::WithLatestFromExt;
