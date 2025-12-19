// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
pub mod tokio;

#[cfg(all(feature = "time-async-std", not(target_arch = "wasm32")))]
pub mod async_std;

#[cfg(all(feature = "time-smol", not(target_arch = "wasm32")))]
pub mod smol;

#[cfg(all(feature = "time-wasm", target_arch = "wasm32"))]
pub mod wasm;
