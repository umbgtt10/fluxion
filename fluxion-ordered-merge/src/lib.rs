// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]

extern crate alloc;

mod ordered_merge;

pub use ordered_merge::{OrderedMerge, OrderedMergeExt};
