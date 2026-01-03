// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub use tokio_impl::tokio_implementation::TokioTimer;

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
mod tokio_impl;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub use async_std_impl::async_std_implementation::AsyncStdTimer;

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
mod async_std_impl;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub use smol_impl::smol_implementation::SmolTimer;

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
mod smol_impl;

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
pub use wasm_impl::wasm_implementation;

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
mod wasm_impl;

#[cfg(feature = "runtime-embassy")]
pub use embassy_impl::embassy_implementation::{EmbassyInstant, EmbassyTimerImpl};

#[cfg(feature = "runtime-embassy")]
mod embassy_impl;
