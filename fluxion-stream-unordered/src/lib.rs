// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Unordered stream operators for Fluxion.
//!
//! This crate provides timestamped stream operators that prioritize performance over
//! temporal ordering. All multi-stream operations use `select_all` for arrival-order
//! processing, resulting in 2-3x speedup compared to ordered variants.

pub mod operators;

pub use operators::combine_latest::CombineLatestExt;
