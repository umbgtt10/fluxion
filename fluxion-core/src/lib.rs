// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod comparable_inner;
pub mod comparable_sync;
pub mod comparable_timestamped;
pub mod comparable_unpin;
pub mod comparable_unpin_timestamped;
pub mod error;
pub mod fluxion_item;
pub mod has_timestamp;
pub mod into_stream;
pub mod lock_utilities;
pub mod ordered_fluxion_item;
pub mod stream_item;
pub mod timestamped;

pub use self::comparable_inner::ComparableInner;
pub use self::comparable_sync::ComparableSync;
pub use self::comparable_timestamped::ComparableTimestamped;
pub use self::comparable_unpin::ComparableUnpin;
pub use self::comparable_unpin_timestamped::ComparableUnpinTimestamped;
pub use self::error::{FluxionError, IntoFluxionError, Result, ResultExt};
pub use self::fluxion_item::FluxionItem;
pub use self::has_timestamp::HasTimestamp;
pub use self::ordered_fluxion_item::OrderedFluxionItem;
pub use self::stream_item::StreamItem;
pub use self::timestamped::Timestamped;
