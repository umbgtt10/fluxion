// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod compare_by_inner;
pub mod error;
pub mod into_stream;
pub mod lock_utilities;
pub mod ordered;
pub mod stream_item;

pub use self::compare_by_inner::CompareByInner;
pub use self::error::{FluxionError, IntoFluxionError, Result, ResultExt};
pub use self::ordered::{Ordered, OrderedWrapper};
pub use self::stream_item::StreamItem;
