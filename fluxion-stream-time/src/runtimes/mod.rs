// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
pub use tokio_impl::tokio_implementation::TokioTimer;

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
mod tokio_impl;

#[cfg(all(feature = "time-async-std", not(target_arch = "wasm32")))]
pub use async_std_impl::async_std_implementation::AsyncStdTimer;

#[cfg(all(feature = "time-async-std", not(target_arch = "wasm32")))]
mod async_std_impl;

#[cfg(all(feature = "time-wasm", target_arch = "wasm32"))]
pub use wasm_impl::wasm_implementation;

#[cfg(all(feature = "time-wasm", target_arch = "wasm32"))]
mod wasm_impl;
