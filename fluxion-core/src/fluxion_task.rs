// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Runtime-agnostic task spawning with cooperative cancellation.
//!
//! This module provides a unified abstraction for spawning background tasks
//! that works across all async runtimes (Tokio, smol, async-std, WASM).

use crate::CancellationToken;
use core::future::Future;

/// Runtime-agnostic task handle with automatic cancellation on drop.
///
/// `FluxionTask` spawns a background task using the configured runtime
/// (Tokio, smol, async-std, or WASM) and provides cooperative cancellation
/// through a `CancellationToken`.
///
/// The spawned task receives a `CancellationToken` that it should monitor
/// to enable graceful shutdown. When the `FluxionTask` is dropped or manually
/// cancelled, the token is signaled, allowing the task to clean up and exit.
///
/// # Runtime Support
///
/// - **Tokio**: `tokio::spawn` (default)
/// - **smol**: `smol::spawn`
/// - **async-std**: `async_core::task::spawn`
/// - **WASM**: `wasm_bindgen_futures::spawn_local`
///
/// Select runtime via feature flags: `runtime-tokio`, `runtime-smol`,
/// `runtime-async-std`, or automatic WASM detection.
///
/// # Example
///
/// ```rust
/// use fluxion_core::FluxionTask;
/// use futures::FutureExt;
///
/// # #[tokio::main]
/// # async fn main() {
/// let task = FluxionTask::spawn(|cancel| async move {
///     loop {
///         if cancel.is_cancelled() {
///             println!("Graceful shutdown");
///             break;
///         }
///         // Do work...
///     }
/// });
///
/// // Task automatically cancels on drop
/// drop(task);
/// # }
/// ```
#[derive(Debug)]
pub struct FluxionTask {
    cancel: CancellationToken,
}

impl FluxionTask {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let _future = f(cancel_clone);

        #[cfg(all(
            feature = "runtime-tokio",
            not(all(feature = "runtime-smol", not(feature = "runtime-tokio"))),
            not(all(
                feature = "runtime-async-std",
                not(feature = "runtime-tokio"),
                not(feature = "runtime-smol")
            ))
        ))]
        tokio::spawn(_future);

        #[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
        smol::spawn(_future).detach();

        #[cfg(all(
            feature = "runtime-async-std",
            not(target_arch = "wasm32"),
            not(feature = "runtime-tokio"),
            not(feature = "runtime-smol")
        ))]
        async_std::task::spawn(_future);

        Self { cancel }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationToken) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let _future = f(cancel_clone);

        wasm_bindgen_futures::spawn_local(_future);

        Self { cancel }
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

impl Drop for FluxionTask {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
