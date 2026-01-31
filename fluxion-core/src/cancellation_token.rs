// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                cancelled: AtomicBool::new(false),
                event: Event::new(),
            }),
        }
    }

    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::Release);
        self.inner.event.notify(usize::MAX);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }

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

pub struct Cancelled<'a> {
    token: &'a CancellationToken,
    listener: Option<EventListener>,
}

impl<'a> Future for Cancelled<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.token.is_cancelled() {
            return Poll::Ready(());
        }

        if self.listener.is_none() {
            self.listener = Some(self.token.inner.event.listen());

            if self.token.is_cancelled() {
                return Poll::Ready(());
            }
        }

        match Pin::new(self.listener.as_mut().unwrap()).poll(cx) {
            Poll::Ready(()) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}
