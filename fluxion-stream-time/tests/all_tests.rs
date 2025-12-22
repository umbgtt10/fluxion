// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Required for #[embassy_executor::task] macro on nightly
#![cfg_attr(feature = "runtime-embassy", feature(impl_trait_in_assoc_type))]

#[cfg(all(
    feature = "runtime-tokio",
    not(feature = "runtime-async-std"),
    not(feature = "runtime-smol"),
    not(target_arch = "wasm32")
))]
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
    not(feature = "runtime-async-std"),
    not(target_arch = "wasm32")
))]
pub mod smol;

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
pub mod wasm;

#[cfg(all(
    feature = "runtime-embassy",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std")
))]
pub mod embassy;
