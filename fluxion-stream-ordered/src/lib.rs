// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Ordered stream operators for Fluxion.
//!
//! This crate provides timestamped stream operators that preserve temporal ordering.
//! All multi-stream operations use `ordered_merge` to maintain causality.

pub mod operators;

pub use operators::combine_latest::CombineLatestExt;
