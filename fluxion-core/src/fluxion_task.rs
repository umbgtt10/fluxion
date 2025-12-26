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
    /// Spawn a background task with cancellation support.
    ///
    /// The provided closure receives a `CancellationToken` that will be triggered
    /// when the task is dropped or manually cancelled. The spawned future should
    /// monitor this token and exit gracefully when cancellation is requested.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that receives a `CancellationToken` and returns a future
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_core::FluxionTask;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let counter = Arc::new(AtomicU32::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let task = FluxionTask::spawn(|cancel| async move {
    ///     while !cancel.is_cancelled() {
    ///         counter_clone.fetch_add(1, Ordering::SeqCst);
    ///     }
    ///     println!("Stopped at count: {}", counter_clone.load(Ordering::SeqCst));
    /// });
    ///
    /// // Do other work...
    ///
    /// // Task cancels automatically on drop
    /// drop(task);
    /// # }
    /// ```
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

    /// Manually cancel the task.
    ///Spawn a background task with cancellation support (WASM version without Send bounds).
    ///
    /// This is the WASM-specific version of `spawn` that does not require `Send` bounds
    /// since WASM is single-threaded. The closure and future only need to be `'static`.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that receives a `CancellationToken` and returns a future
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

    ///
    /// This signals the task to stop but doesn't wait for it to complete.
    /// The task will stop at its next cancellation checkpoint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_core::FluxionTask;
    ///
    /// # async fn example() {
    /// let task = FluxionTask::spawn(|cancel| async move {
    ///     loop {
    ///         if cancel.is_cancelled() {
    ///             break;
    ///         }
    ///         // Do work...
    ///     }
    /// });
    ///
    /// // Explicitly cancel before drop
    /// task.cancel();
    /// # }
    /// ```
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Check if cancellation has been requested.
    ///
    /// Returns `true` if the task has been cancelled via `cancel()` or drop.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_core::FluxionTask;
    ///
    /// # async fn example() {
    /// let task = FluxionTask::spawn(|cancel| async move {
    ///     // Task body...
    /// });
    ///
    /// assert!(!task.is_cancelled());
    /// task.cancel();
    /// assert!(task.is_cancelled());
    /// # }
    /// ```
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

impl Drop for FluxionTask {
    fn drop(&mut self) {
        // Signal cancellation to allow graceful shutdown
        self.cancel.cancel();
    }
}
