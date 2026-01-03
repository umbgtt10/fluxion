// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt::Debug;
use fluxion_core::StreamItem;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use futures::Stream;
use std::time::Duration;
use tokio::select;
use tokio::time::sleep;

/// Receives a value from an UnboundedReceiver with a timeout.
///
/// # Panics
/// Panics if no item is received within the timeout.
pub async fn recv_timeout<T>(rx: &mut UnboundedReceiver<T>, timeout_ms: u64) -> Option<T> {
    select! {
        item = rx.next() => item,
        () = sleep(Duration::from_millis(timeout_ms)) => {
            panic!("Timeout: No item received within {} ms", timeout_ms)
        }
    }
}

/// Asserts that no item is received from an UnboundedReceiver within a timeout.
///
/// # Panics
/// Panics if an item is received within the timeout.
pub async fn assert_no_recv<T>(rx: &mut UnboundedReceiver<T>, timeout_ms: u64) {
    select! {
        item = rx.next() => {
            if item.is_some() { panic!("Unexpected item received within {} ms", timeout_ms) }
        }
        () = sleep(Duration::from_millis(timeout_ms)) => {
            // Timeout occurred, which is success
        }
    }
}

/// Unwraps a `StreamItem::Value`, panicking if it's an error.
///
/// This helper eliminates the common `.unwrap().unwrap()` pattern in tests
/// where you need to extract the value from both the `Option` and `StreamItem`.
///
/// # Panics
///
/// Panics if the `StreamItem` is an `Error` variant.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, unwrap_value, unwrap_stream, Sequenced};
/// use fluxion_test_utils::test_data::person_alice;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel();
/// tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
///
/// // Instead of: let item = stream.next().await.unwrap().unwrap();
/// // Prefer the async helper which waits safely for spawned tasks:
/// let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
/// # }
/// ```
pub fn unwrap_value<T>(item: Option<StreamItem<T>>) -> T {
    match item {
        Some(StreamItem::Value(value)) => value,
        Some(StreamItem::Error(e)) => panic!("Expected Value but got Error: {}", e),
        None => panic!("Expected Value but stream ended"),
    }
}

/// Unwraps a value from a stream with a timeout for spawned tasks to process.
///
/// This function polls the stream with a timeout to allow spawned background tasks
/// time to process items. This is useful when testing streams that use `tokio::spawn`
/// internally.
///
/// # Panics
///
/// Panics if:
/// - The stream ends (returns `None`) before an item arrives
/// - No item is received within the 500ms timeout
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, unwrap_stream, Sequenced};
/// use fluxion_test_utils::test_data::person_alice;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel();
/// tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
///
/// // Waits up to 500ms for the item to arrive
/// let item = unwrap_stream(&mut stream, 500).await;
/// # }
/// ```
pub async fn unwrap_stream<T, S>(stream: &mut S, timeout_ms: u64) -> StreamItem<T>
where
    S: Stream<Item = StreamItem<T>> + Unpin,
{
    select! {
        item = stream.next() => {
            item.expect("Expected StreamItem but stream ended")
        }
        () = sleep(Duration::from_millis(timeout_ms)) => {
            panic!("Timeout: No item received within {} ms", timeout_ms)
        }
    }
}

/// Creates a test channel that automatically wraps values in `StreamItem::Value`.
///
/// This helper simplifies test setup by handling the `StreamItem` wrapping automatically,
/// allowing tests to send plain values while the stream receives `StreamItem<T>`.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, Sequenced};
/// use fluxion_test_utils::test_data::person_alice;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel();
///
/// // Send plain values
/// tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
///
/// // Receive StreamItem-wrapped values (prefer using `unwrap_stream` in async tests)
/// // Option -> StreamItem -> Value
/// // Example using the async helper:
/// // let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
/// # }
/// ```
pub fn test_channel<T: Send + 'static>(
) -> (UnboundedSender<T>, impl Stream<Item = StreamItem<T>> + Send) {
    let (tx, rx) = unbounded();
    let stream = rx.map(StreamItem::Value);
    (tx, stream)
}

/// Creates a test channel that accepts `StreamItem<T>` for testing error propagation.
///
/// This helper allows tests to explicitly send both values and errors through the stream,
/// enabling comprehensive error handling tests.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::test_channel_with_errors;
/// use fluxion_core::{StreamItem, FluxionError};
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel_with_errors();
///
/// // Send values
/// tx.unbounded_send(StreamItem::Value(42)).unwrap();
///
/// // Send errors
/// tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error"))).unwrap();
///
/// let value = stream.next().await.unwrap();
/// let error = stream.next().await.unwrap();
/// # }
/// ```
pub fn test_channel_with_errors<T: Send + 'static>() -> (
    UnboundedSender<StreamItem<T>>,
    impl Stream<Item = StreamItem<T>> + Send,
) {
    let (tx, rx) = unbounded();
    (tx, rx)
}

/// Assert that no element is emitted within the given timeout.
///
/// # Panics
/// Panics if the stream emits an element before the timeout elapses.
pub async fn assert_no_element_emitted<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: Stream<Item = T> + Send + Unpin,
    T: Debug,
{
    select! {
        state = stream.next() => {
            panic!(
                "Unexpected combination emitted {:?}, expected no output.",
                state
            );
        }
        () = sleep(Duration::from_millis(timeout_ms)) => {
        }
    }
}

/// Assert that the stream has ended (returns None) within a timeout.
///
/// This prevents tests from hanging when checking if a stream has terminated.
/// If the stream doesn't end within the timeout, the test will panic.
///
/// # Panics
/// Panics if:
/// - The stream returns a value instead of None
/// - The stream doesn't end within the timeout
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, assert_stream_ended};
/// # async fn example() {
/// let (tx, mut stream) = test_channel::<i32>();
/// drop(tx); // Close the stream
///
/// // This will pass because the stream has ended
/// assert_stream_ended(&mut stream, 500).await;
/// # }
/// ```
pub async fn assert_stream_ended<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: Stream<Item = T> + Unpin,
{
    select! {
        item = stream.next() => {
            if item.is_some() { panic!("Expected stream to end but it returned a value") }
        }
        () = sleep(Duration::from_millis(timeout_ms)) => {
            panic!("Timeout: Stream did not end within {} ms", timeout_ms)
        }
    }
}

/// Collects all values from a stream until it times out waiting for the next item.
///
/// This function repeatedly polls the stream with a per-item timeout. It collects
/// all `StreamItem::Value` items, ignoring errors, until no more items arrive within
/// the timeout period.
///
/// # Returns
/// A `Vec<T>` containing all the inner values from `StreamItem::Value` items.
///
/// # Example
///
/// ```rust
/// use fluxion_test_utils::{test_channel, unwrap_all, Sequenced};
/// use fluxion_test_utils::test_data::{person_alice, person_bob};
///
/// # async fn example() {
/// let (tx, mut stream) = test_channel();
/// tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
/// tx.unbounded_send(Sequenced::new(person_bob())).unwrap();
/// drop(tx);
///
/// let results = unwrap_all(&mut stream, 100).await;
/// assert_eq!(results.len(), 2);
/// # }
/// ```
pub async fn unwrap_all<T, S>(stream: &mut S, timeout_ms: u64) -> Vec<T>
where
    S: Stream<Item = StreamItem<T>> + Unpin,
{
    let mut results = Vec::new();
    loop {
        let item = select! {
            item = stream.next() => item,
            () = sleep(Duration::from_millis(timeout_ms)) => break,
        };

        match item {
            Some(StreamItem::Value(val)) => results.push(val),
            Some(StreamItem::Error(_)) => continue, // Ignore errors
            None => break,                          // Stream ended
        }
    }
    results
}

/// Macro to wrap test bodies with timeout to prevent hanging tests
#[macro_export]
macro_rules! with_timeout {
    ($test_body:expr) => {
        timeout(Duration::from_secs(5), async { $test_body })
            .await
            .expect("Test timed out after 5 seconds")
    };
}
