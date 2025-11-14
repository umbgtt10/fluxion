// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod compare_by_inner;
pub mod into_stream;
pub mod ordered;

pub use self::compare_by_inner::CompareByInner;
pub use self::ordered::{Ordered, OrderedWrapper};
