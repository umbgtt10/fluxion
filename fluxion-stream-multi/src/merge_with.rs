// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Re-export of `MergedStream` for single-threaded runtimes.
//!
//! A stateful stream merger that combines multiple Timestamped streams while maintaining state.

pub use fluxion_stream_core::merge_with::MergedStream;
