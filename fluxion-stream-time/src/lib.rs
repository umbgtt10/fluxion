// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Time-based operators for streams using std::time::Instant timestamps.
//!
//! This crate provides time-based operators for delaying and debouncing stream emissions,
//! along with the `InstantTimestamped<T>` wrapper type that uses `std::time::Instant` for timestamps.
//!
//! # Overview
//!
//! - **`InstantTimestamped<T>`** - Wraps a value with a monotonic Instant timestamp
//! - **`DelayExt`** - Extension trait for `.delay(duration)`
//! - **`DebounceExt`** - Extension trait for `.debounce(duration)`
//! - **`ThrottleExt`** - Extension trait for `.throttle(duration)`
//! - **`SampleExt`** - Extension trait for `.sample(duration)`
//! - **`TimeoutExt`** - Extension trait for `.timeout(duration)`
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream_time::prelude::*;
//! use fluxion_stream_time::InstantTimestamped;
//! use fluxion_core::StreamItem;
//! use futures::stream::StreamExt;
//! use std::time::Duration;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//!
//! # async fn example() {
//! let (tx, rx) = mpsc::unbounded_channel();
//!
//! // Delay all emissions by 100ms
//! let delayed = UnboundedReceiverStream::new(rx)
//!     .map(StreamItem::Value)
//!     .delay(Duration::from_millis(100));
//!
//! tx.send(InstantTimestamped::now(42)).unwrap();
//! tx.send(InstantTimestamped::now(100)).unwrap();
//!
//! // Or debounce to emit only after 100ms of quiet time
//! # let (tx, rx) = mpsc::unbounded_channel();
//! # tx.send(InstantTimestamped::now(42)).unwrap();
//! let debounced = UnboundedReceiverStream::new(rx)
//!     .map(StreamItem::Value)
//!     .debounce(Duration::from_millis(100));
//! # }
//! ```

mod debounce;
mod delay;
mod instant_timestamped;
mod sample;
mod throttle;
mod timeout;

pub mod prelude;

pub use debounce::DebounceExt;
pub use delay::DelayExt;
pub use instant_timestamped::InstantTimestamped;
pub use sample::SampleExt;
pub use throttle::ThrottleExt;
pub use timeout::TimeoutExt;
