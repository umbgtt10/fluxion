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
//! - `runtime-tokio` (default) - Tokio runtime support with `TokioTimer`
//! - `runtime-smol` - smol runtime support with `SmolTimer`
//! - `runtime-wasm` - WebAssembly support with `WasmTimer`
//! - `runtime-embassy` - Embassy (embedded) runtime support with `EmbassyTimerImpl` (no_std)
//! - `runtime-async-std` - async-std runtime ⚠️ **DEPRECATED** (unmaintained, RUSTSEC-2025-0052)
//!
//! ⚠️ **Note**: async-std is discontinued. Use tokio or smol for new projects.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(not(target_arch = "wasm32"))]
//! use fluxion_stream_time::prelude::*;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{TokioTimestamped, TokioTimer};
//! use fluxion_stream_time::timer::Timer;
//! use fluxion_core::StreamItem;
//! use futures::stream::StreamExt;
//! use std::time::Duration;
//! use futures::channel::mpsc;
//!
//! # #[cfg(not(target_arch = "wasm32"))]
//! # async fn example() {
//! let (tx, rx) = mpsc::unbounded::<TokioTimestamped<i32>>();
//! let timer = TokioTimer;
//!
//! // Delay all emissions by 100ms (convenience method)
//! let delayed = rx
//!     .map(StreamItem::Value)
//!     .delay(Duration::from_millis(100));
//!
//! tx.unbounded_send(TokioTimestamped::new(42, timer.now())).unwrap();
//! tx.unbounded_send(TokioTimestamped::new(100, timer.now())).unwrap();
//!
//! // Or debounce with convenience method (no timer parameter needed)
//! # let (tx, rx) = mpsc::unbounded::<TokioTimestamped<i32>>();
//! let debounced = rx
//!     .map(StreamItem::Value)
//!     .debounce(Duration::from_millis(100));
//! # }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod debounce;
mod delay;
mod instant_timestamped;
pub mod runtimes;
mod sample;
mod throttle;
mod timeout;
pub mod timer;

pub mod prelude;

pub use debounce::{DebounceExt, DebounceWithDefaultTimerExt};
pub use delay::{DelayExt, DelayWithDefaultTimerExt};
pub use instant_timestamped::InstantTimestamped;
pub use sample::{SampleExt, SampleWithDefaultTimerExt};
pub use throttle::{ThrottleExt, ThrottleWithDefaultTimerExt};
pub use timeout::{TimeoutExt, TimeoutWithDefaultTimerExt};

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub use runtimes::TokioTimer;

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub type TokioTimestamped<T> = InstantTimestamped<T, TokioTimer>;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub use runtimes::AsyncStdTimer;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub type AsyncStdTimestamped<T> = InstantTimestamped<T, AsyncStdTimer>;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub use runtimes::SmolTimer;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub type SmolTimestamped<T> = InstantTimestamped<T, SmolTimer>;

#[cfg(feature = "runtime-embassy")]
pub use runtimes::EmbassyTimerImpl;

#[cfg(feature = "runtime-embassy")]
pub type EmbassyTimestamped<T> = InstantTimestamped<T, EmbassyTimerImpl>;
