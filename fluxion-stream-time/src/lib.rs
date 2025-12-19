// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Time-based operators for streams with runtime-agnostic timer abstraction.
//!
//! This crate provides time-based operators for delaying, debouncing, throttling, sampling,
//! and timeout handling of stream emissions. Operators work with any async runtime through
//! the `Timer` trait abstraction.
//!
//! # Overview
//!
//! - **`Timer` trait** - Runtime-agnostic timer abstraction
//! - **`InstantTimestamped<T, TM>`** - Wraps a value with a Timer's Instant timestamp
//! - **`DelayExt`** - Extension trait for `.delay(duration, timer)`
//! - **`DebounceExt`** - Extension trait for `.debounce(duration, timer)`
//! - **`ThrottleExt`** - Extension trait for `.throttle(duration, timer)`
//! - **`SampleExt`** - Extension trait for `.sample(duration, timer)`
//! - **`TimeoutExt`** - Extension trait for `.timeout(duration, timer)`
//!
//! # Runtime Support
//!
//! Enable runtime-specific features in your `Cargo.toml`:
//! - `time-tokio` (default) - Tokio runtime support with `TokioTimer`
//! - `time-async-std` - async-std runtime ⚠️ **DEPRECATED** (unmaintained, RUSTSEC-2025-0052)
//! - `time-wasm` - WebAssembly support with `WasmTimer`
//! - `time-smol` - smol runtime support (planned)
//!
//! ⚠️ **Note**: async-std is discontinued. Use tokio for new projects.
//!
//! # Example
//!
//! ```rust
//! use fluxion_stream_time::prelude::*;
//! use fluxion_stream_time::{TokioTimestamped, TokioTimer};
//! use fluxion_stream_time::timer::Timer;
//! use fluxion_core::StreamItem;
//! use futures::stream::StreamExt;
//! use std::time::Duration;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//!
//! # async fn example() {
//! let (tx, rx) = mpsc::unbounded_channel::<TokioTimestamped<i32>>();
//! let timer = TokioTimer;
//!
//! // Delay all emissions by 100ms
//! let delayed = UnboundedReceiverStream::new(rx)
//!     .map(StreamItem::Value)
//!     .delay(Duration::from_millis(100), timer.clone());
//!
//! tx.send(TokioTimestamped::new(42, timer.now())).unwrap();
//! tx.send(TokioTimestamped::new(100, timer.now())).unwrap();
//!
//! // Or debounce to emit only after 100ms of quiet time
//! # let (tx, rx) = mpsc::unbounded_channel::<TokioTimestamped<i32>>();
//! # let timer = TokioTimer;
//! let debounced = UnboundedReceiverStream::new(rx)
//!     .map(StreamItem::Value)
//!     .debounce(Duration::from_millis(100), timer);
//! # }
//! ```

mod debounce;
mod delay;
mod instant_timestamped;
pub mod runtimes;
mod sample;
mod throttle;
mod timeout;
pub mod timer;

pub mod prelude;

pub use debounce::DebounceExt;
pub use delay::DelayExt;
pub use instant_timestamped::InstantTimestamped;
pub use sample::SampleExt;
pub use throttle::ThrottleExt;
pub use timeout::TimeoutExt;

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
pub use runtimes::TokioTimer;

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
pub type TokioTimestamped<T> = InstantTimestamped<T, TokioTimer>;

#[cfg(all(feature = "time-async-std", not(target_arch = "wasm32")))]
pub use runtimes::AsyncStdTimer;

#[cfg(all(feature = "time-async-std", not(target_arch = "wasm32")))]
pub type AsyncStdTimestamped<T> = InstantTimestamped<T, AsyncStdTimer>;
