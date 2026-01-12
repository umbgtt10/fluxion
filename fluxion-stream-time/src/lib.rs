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
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_stream_time::{DebounceExt, DelayExt, TokioTimestamped};
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::impls::tokio::TokioTimer;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_runtime::timer::Timer;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use fluxion_core::StreamItem;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use futures::stream::StreamExt;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use std::time::Duration;
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
//! use futures::channel::mpsc;
//!
//! # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
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
//! # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
//! # fn example() {}
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod debounce;
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod throttle;

mod delay;
mod instant_timestamped;

mod sample;
mod timeout;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
pub use debounce::DebounceExt;
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
pub use throttle::ThrottleExt;

pub use delay::DelayExt;
pub use instant_timestamped::InstantTimestamped;
pub use sample::SampleExt;
pub use timeout::TimeoutExt;

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub use fluxion_runtime::impls::tokio::TokioRuntime;

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub type TokioTimestamped<T> = InstantTimestamped<T, TokioRuntime>;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub use fluxion_runtime::impls::async_std::AsyncStdRuntime;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub type AsyncStdTimestamped<T> = InstantTimestamped<T, AsyncStdRuntime>;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub use fluxion_runtime::impls::smol::SmolRuntime;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub type SmolTimestamped<T> = InstantTimestamped<T, SmolRuntime>;

#[cfg(all(
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std"),
    not(feature = "runtime-embassy"),
    feature = "runtime-wasm"
))]
pub use fluxion_runtime::impls::wasm::WasmRuntime;

#[cfg(all(
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std"),
    not(feature = "runtime-embassy"),
    feature = "runtime-wasm"
))]
pub type WasmTimestamped<T> = InstantTimestamped<T, WasmRuntime>;

#[cfg(feature = "runtime-embassy")]
pub use fluxion_runtime::impls::embassy::EmbassyRuntime;

#[cfg(feature = "runtime-embassy")]
pub type EmbassyTimestamped<T> = InstantTimestamped<T, EmbassyRuntime>;

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub type DefaultRuntime = fluxion_runtime::impls::tokio::TokioRuntime;

#[cfg(all(
    not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))),
    feature = "runtime-smol"
))]
pub type DefaultRuntime = fluxion_runtime::impls::smol::SmolRuntime;

#[cfg(all(
    not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))),
    not(feature = "runtime-smol"),
    feature = "runtime-async-std"
))]
pub type DefaultRuntime = fluxion_runtime::impls::async_std::AsyncStdRuntime;

#[cfg(all(
    not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std"),
    feature = "runtime-embassy"
))]
pub type DefaultRuntime = fluxion_runtime::impls::embassy::EmbassyRuntime;

#[cfg(all(
    not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std"),
    not(feature = "runtime-embassy"),
    feature = "runtime-wasm"
))]
pub type DefaultRuntime = fluxion_runtime::impls::wasm::WasmRuntime;
