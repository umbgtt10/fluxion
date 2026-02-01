// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg_attr(not(feature = "std"), no_std)]

mod instant_timestamped;
pub use instant_timestamped::InstantTimestamped;

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
pub use debounce::DebounceExt;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod delay;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
pub use delay::DelayExt;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod sample;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
pub use sample::SampleExt;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod throttle;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
pub use throttle::ThrottleExt;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
mod timeout;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
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
