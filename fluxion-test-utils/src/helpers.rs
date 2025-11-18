// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::sequenced::Sequenced;
use crate::test_data::TestData;
use fluxion_core::{FluxionError, Result, StreamItem};
use futures::stream::StreamExt;
use futures::Stream;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::sleep;
use tokio_stream::wrappers::UnboundedReceiverStream;

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
/// tx.send(Sequenced::new(person_alice())).unwrap();
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
/// tx.send(Sequenced::new(person_alice())).unwrap();
///
/// // Waits up to 500ms for the item to arrive
/// let item = unwrap_stream(&mut stream, 500).await;
/// # }
/// ```
pub async fn unwrap_stream<T, S>(stream: &mut S, timeout_ms: u64) -> StreamItem<T>
where
    S: futures::stream::Stream<Item = StreamItem<T>> + Unpin,
{
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    match timeout(Duration::from_millis(timeout_ms), stream.next()).await {
        Ok(Some(item)) => item,
        Ok(None) => panic!("Expected StreamItem but stream ended"),
        Err(_) => panic!("Timeout: No item received within 500ms"),
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
/// tx.send(Sequenced::new(person_alice())).unwrap();
///
/// // Receive StreamItem-wrapped values (prefer using `unwrap_stream` in async tests)
/// // Option -> StreamItem -> Value
/// // Example using the async helper:
/// // let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
/// # }
/// ```
pub fn test_channel<T: Send + 'static>(
) -> (UnboundedSender<T>, impl Stream<Item = StreamItem<T>> + Send) {
    let (tx, rx) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx).map(StreamItem::Value);
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
/// tx.send(StreamItem::Value(42)).unwrap();
///
/// // Send errors
/// tx.send(StreamItem::Error(FluxionError::stream_error("test error"))).unwrap();
///
/// let value = stream.next().await.unwrap();
/// let error = stream.next().await.unwrap();
/// # }
/// ```
pub fn test_channel_with_errors<T: Send + 'static>() -> (
    UnboundedSender<StreamItem<T>>,
    impl Stream<Item = StreamItem<T>> + Send,
) {
    let (tx, rx) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx);
    (tx, stream)
}

/// Assert that no element is emitted within the given timeout.
///
/// # Panics
/// Panics if the stream emits an element before the timeout elapses.
pub async fn assert_no_element_emitted<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: Stream<Item = T> + Unpin,
{
    tokio::select! {
        _state = stream.next() => {
            panic!(
                "Unexpected combination emitted, expected no output."
            );
        }
        () = sleep(Duration::from_millis(timeout_ms)) => {
        }
    }
}

/// Expect the next value from a stream, returning an error if none available
///
/// # Errors
/// Returns an error if the stream ends before yielding the next item or the value mismatches.
pub async fn expect_next_value<S>(stream: &mut S, expected: TestData) -> Result<()>
where
    S: Stream<Item = TestData> + Unpin,
{
    let item = stream
        .next()
        .await
        .ok_or_else(|| FluxionError::stream_error("Expected next item but stream ended"))?;

    if item == expected {
        Ok(())
    } else {
        Err(FluxionError::stream_error(format!(
            "Expected {expected:?}, got {item:?}"
        )))
    }
}

/// Expect the next value from a stream, panicking if none available (for backward compatibility)
///
/// # Panics
/// Panics if the stream ends before yielding the next item.
pub async fn expect_next_value_unchecked<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = TestData> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item, expected);
}

/// Expect the next timestamped value from a stream, returning an error if none available
///
/// # Errors
/// Returns an error if the stream ends before yielding the next item or the value mismatches.
pub async fn expect_next_timestamped<S>(stream: &mut S, expected: TestData) -> Result<()>
where
    S: Stream<Item = Sequenced<TestData>> + Unpin,
{
    let item = stream.next().await.ok_or_else(|| {
        FluxionError::stream_error("Expected next timestamped item but stream ended")
    })?;

    if item.value == expected {
        Ok(())
    } else {
        Err(FluxionError::stream_error(format!(
            "Expected {expected:?}, got {:?}",
            item.value
        )))
    }
}

/// Expect the next timestamped value from a stream, panicking if none available (for backward compatibility)
///
/// # Panics
/// Panics if the stream ends before yielding the next item.
pub async fn expect_next_timestamped_unchecked<S>(stream: &mut S, expected: TestData)
where
    S: Stream<Item = Sequenced<TestData>> + Unpin,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item.value, expected);
}

/// Expect the next pair from a `with_latest_from` stream, returning an error if none available
///
/// # Errors
/// Returns an error if the stream ends before yielding the next pair or the values mismatch.
pub async fn expect_next_pair<S>(
    stream: &mut S,
    expected_left: TestData,
    expected_right: TestData,
) -> Result<()>
where
    S: Stream<Item = (Sequenced<TestData>, Sequenced<TestData>)> + Unpin,
{
    let (left, right) = stream
        .next()
        .await
        .ok_or_else(|| FluxionError::stream_error("Expected next pair but stream ended"))?;

    if left.value == expected_left && right.value == expected_right {
        Ok(())
    } else {
        Err(FluxionError::stream_error(format!(
            "Expected ({expected_left:?}, {expected_right:?}), got ({:?}, {:?})",
            left.value, right.value
        )))
    }
}

/// Expect the next pair from a `with_latest_from` stream, panicking if none available (for backward compatibility)
///
/// # Panics
/// Panics if the stream ends before yielding the next pair.
pub async fn expect_next_pair_unchecked<S>(
    stream: &mut S,
    expected_left: TestData,
    expected_right: TestData,
) where
    S: Stream<Item = (Sequenced<TestData>, Sequenced<TestData>)> + Unpin,
{
    let (left, right) = stream.next().await.expect("expected next pair");
    assert_eq!((left.value, right.value), (expected_left, expected_right));
}

/// Macro to wrap test bodies with timeout to prevent hanging tests
#[macro_export]
macro_rules! with_timeout {
    ($test_body:expr) => {
        tokio::time::timeout(std::time::Duration::from_secs(5), async { $test_body })
            .await
            .expect("Test timed out after 5 seconds")
    };
}
