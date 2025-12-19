// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Prelude module re-exporting all commonly used traits and types.
//!
//! Import this module for convenient access to all time-based stream operators:
//!
//! ```ignore
//! use fluxion_stream_time::prelude::*;
//!
//! // Now all extension traits are available
//! let processed = stream
//!     .delay(Duration::from_millis(100))
//!     .debounce(Duration::from_millis(50))
//!     .throttle(Duration::from_millis(200));
//! ```
//!
//! # Contents
//!
//! ## Extension Traits (Operators)
//!
//! - [`DelayExt`] - Delay each emission by a duration
//! - [`DebounceExt`] - Emit only after a quiet period
//! - [`ThrottleExt`] - Emit then ignore for a duration
//! - [`SampleExt`] - Emit most recent value at intervals
//! - [`TimeoutExt`] - Error if no emission within duration
//!
//! ## Types
//!
//! - [`InstantTimestamped`] - Wrapper with monotonic Instant timestamp

// For debounce, export both traits - they don't conflict since methods have different names
pub use crate::debounce::{DebounceExt, DebounceWithDefaultTimerExt};
pub use crate::delay::{DelayExt, DelayWithDefaultTimerExt};
pub use crate::sample::{SampleExt, SampleWithDefaultTimerExt};
pub use crate::throttle::{ThrottleExt, ThrottleWithDefaultTimerExt};
pub use crate::timeout::{TimeoutExt, TimeoutWithDefaultTimerExt};

pub use crate::InstantTimestamped;
