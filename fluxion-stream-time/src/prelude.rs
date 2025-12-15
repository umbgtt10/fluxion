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
//! - [`ChronoTimestamped`] - Wrapper with UTC timestamp

pub use crate::debounce::DebounceExt;
pub use crate::delay::DelayExt;
pub use crate::sample::SampleExt;
pub use crate::throttle::ThrottleExt;
pub use crate::timeout::TimeoutExt;

pub use crate::ChronoTimestamped;
