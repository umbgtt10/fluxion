// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        tracing::error!($($arg)*);
    }};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        tracing::warn!($($arg)*);
    }};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        tracing::info!($($arg)*);
    }};
}

// With std available (runtime features enabled)
#[cfg(all(
    not(feature = "tracing"),
    any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    )
))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
    }};
}

#[cfg(all(
    not(feature = "tracing"),
    any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    )
))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
    }};
}

#[cfg(all(
    not(feature = "tracing"),
    any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    )
))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}

// In no_std mode (no runtime features), logging is a no-op
#[cfg(all(
    not(feature = "tracing"),
    not(any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    ))
))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{}};
}

#[cfg(all(
    not(feature = "tracing"),
    not(any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    ))
))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{}};
}

#[cfg(all(
    not(feature = "tracing"),
    not(any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    ))
))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{}};
}
