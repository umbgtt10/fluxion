// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! # Fluxion Stream - Runtime-Agnostic Reactive Streams
//!
//! This is a facade crate that provides a unified API for reactive stream operators
//! across different async runtimes. The actual implementation is delegated to either
//! `fluxion-stream-multi` or `fluxion-stream-single` based on feature selection.
//!
//! ## Feature Selection
//!
//! Choose the appropriate feature for your runtime:
//!
//! ### Multi-threaded Runtimes (default)
//! ```toml
//! fluxion-stream = { version = "0.7", features = ["multi"] }
//! # or convenience aliases:
//! fluxion-stream = { version = "0.7", features = ["tokio"] }
//! fluxion-stream = { version = "0.7", features = ["smol"] }
//! fluxion-stream = { version = "0.7", features = ["async-std"] }
//! ```
//!
//! Provides full operator set including:
//! - All stream operators (map, filter, scan, combine_latest, ordered_merge, etc.)
//! - Background task operators (share, partition, subscribe_latest)
//! - Requires `std` environment
//!
//! ### Single-threaded Runtimes (no_std)
//! ```toml
//! fluxion-stream = { version = "0.7", default-features = false, features = ["single"] }
//! # or convenience aliases:
//! fluxion-stream = { version = "0.7", default-features = false, features = ["embassy"] }
//! fluxion-stream = { version = "0.7", default-features = false, features = ["wasm"] }
//! ```
//!
//! Provides core operator set:
//! - All stream operators (map, filter, scan, combine_latest, ordered_merge, etc.)
//! - ❌ Excludes: share, partition, subscribe_latest (require background tasks)
//! - Works in `no_std` environments with `alloc`
//!
//! ## Usage
//!
//! ```rust
//! use fluxion_stream::prelude::*;
//!
//! // Use the operators - same API regardless of runtime!
//! // stream
//! //     .map_ordered(|x| x * 2)
//! //     .filter_ordered(|x| x > 10)
//! //     .ordered_merge(other_stream)
//! ```
//!
//! ## Architecture
//!
//! This facade enables:
//! - ✅ **Unified API**: Same operator names and signatures across runtimes
//! - ✅ **Runtime flexibility**: Switch runtimes by changing a feature flag
//! - ✅ **Unlimited chaining**: Mix time-bound and non-time-bound operators freely
//! - ✅ **Zero overhead**: No runtime abstraction cost, direct delegation

#![cfg_attr(not(feature = "multi"), no_std)]
#![allow(clippy::multiple_crate_versions)]

// Re-export everything from the selected implementation
#[cfg(feature = "multi")]
pub use fluxion_stream_multi::*;

#[cfg(feature = "single")]
pub use fluxion_stream_single::*;

// Re-export core types for test compatibility
// These are available during testing (both unit and integration tests)
#[cfg(test)]
pub use fluxion_core::{
    fluxion_mutex, into_stream, FluxionError, FluxionSubject, HasTimestamp, StreamItem,
    SubjectError, Timestamped,
};

// Compatibility types module for tests
// Make available for integration tests by not gating it
pub mod types {
    pub use super::{CombinedState, WithPrevious};
}

// Ensure only one feature is selected
#[cfg(all(feature = "multi", feature = "single"))]
compile_error!("Cannot enable both 'multi' and 'single' features. Choose one runtime type.");

#[cfg(not(any(feature = "multi", feature = "single")))]
compile_error!("Must enable either 'multi' or 'single' feature. Use default features for 'multi' or specify 'single' for no_std.");

/// Prelude module for convenient importing of all traits.
///
/// Import this with:
/// ```rust
/// use fluxion_stream::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "multi")]
    pub use fluxion_stream_multi::prelude::*;

    #[cfg(feature = "single")]
    pub use fluxion_stream_single::prelude::*;
}

// End of facade - all operator implementations are in fluxion-stream-multi and fluxion-stream-single
