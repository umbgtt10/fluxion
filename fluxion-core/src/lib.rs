// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod ordered;

// Re-export commonly used types
pub use ordered::{Ordered, OrderedWrapper};

/// Trait for comparing ordered types by their inner values.
/// This is used by stream operators to establish a stable ordering.
pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering;
}
