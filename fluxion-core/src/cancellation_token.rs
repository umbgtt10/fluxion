// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Runtime-agnostic cancellation token.
//!
//! This module provides a drop-in replacement for `tokio_util::sync::CancellationToken`
//! that works on any async runtime (Tokio, smol, async-std, WASM).

use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use event_listener::{Event, EventListener};

/// Runtime-agnostic cancellation token.
///
/// A `CancellationToken` can be cloned to create multiple handles to the same
/// cancellation state. When `cancel()` is called on any clone, all waiters on
/// `cancelled()` will be notified.
///
/// # Example
///
/// ```
/// use fluxion_core::CancellationToken;
///
/// # async fn example() {
/// let token = CancellationToken::new();
/// let token_clone = token.clone();
///
/// tokio::spawn(async move {
///     token_clone.cancelled().await;
///     println!("Cancelled!");
/// });
///
/// // Cancel from another task
/// token.cancel();
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct CancellationToken {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    cancelled: AtomicBool,
    event: Event,
}

impl CancellationToken {
    /// Create a new cancellation token.
    ///
    /// The token is initially not cancelled.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                cancelled: AtomicBool::new(false),
                event: Event::new(),
            }),
        }
    }

    /// Cancel the token, waking all listeners.
    ///
    /// This method is idempotent. Calling it multiple times has the same effect
    /// as calling it once.
    pub fn cancel(&self) {
        // Set flag first with release ordering to ensure all writes are visible
        // before notifying waiters
        self.inner.cancelled.store(true, Ordering::Release);

        // Wake ALL waiters (unbounded notification)
        self.inner.event.notify(usize::MAX);
    }

    /// Check if the token has been cancelled (non-blocking).
    ///
    /// # Example
    ///
    /// ```
    /// use fluxion_core::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// assert!(!token.is_cancelled());
    ///
    /// token.cancel();
    /// assert!(token.is_cancelled());
    /// ```
    pub fn is_cancelled(&self) -> bool {
        // Acquire ordering to see all writes that happened before cancel()
        self.inner.cancelled.load(Ordering::Acquire)
    }

    /// Wait asynchronously until the token is cancelled.
    ///
    /// If the token is already cancelled, this returns immediately.
    ///
    /// # Example
    ///
    /// ```
    /// use fluxion_core::CancellationToken;
    ///
    /// # async fn example() {
    /// let token = CancellationToken::new();
    /// let token_clone = token.clone();
    ///
    /// tokio::spawn(async move {
    ///     token_clone.cancelled().await;
    ///     // Continue after cancellation
    /// });
    ///
    /// token.cancel();
    /// # }
    /// ```
    pub fn cancelled(&self) -> Cancelled<'_> {
        Cancelled {
            token: self,
            listener: None,
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Future returned by [`CancellationToken::cancelled()`].
///
/// This future resolves when the token is cancelled.
pub struct Cancelled<'a> {
    token: &'a CancellationToken,
    listener: Option<EventListener>,
}

impl<'a> Future for Cancelled<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // Fast path: check if already cancelled
        if self.token.is_cancelled() {
            return Poll::Ready(());
        }

        // Register listener if not yet done
        if self.listener.is_none() {
            self.listener = Some(self.token.inner.event.listen());

            // Check again after registering to avoid race condition:
            // If cancel() was called between the first check and listen(),
            // we need to return Ready immediately
            if self.token.is_cancelled() {
                return Poll::Ready(());
            }
        }

        // Poll the listener
        match Pin::new(self.listener.as_mut().unwrap()).poll(cx) {
            Poll::Ready(()) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}
