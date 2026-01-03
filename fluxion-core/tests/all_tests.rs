// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
pub mod tokio;

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(target_arch = "wasm32")
))]
pub mod async_std;

#[cfg(all(
    feature = "runtime-smol",
    not(feature = "runtime-tokio"),
    not(target_arch = "wasm32")
))]
pub mod smol;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
