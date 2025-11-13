use crate::sequenced::Sequenced;
use crate::test_data::TestData;
use fluxion_error::{FluxionError, Result};
use futures::Stream;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

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
        .ok_or_else(|| FluxionError::InvalidState {
            message: "Expected next item but stream ended".to_string(),
        })?;

    if item == expected {
        Ok(())
    } else {
        Err(FluxionError::InvalidState {
            message: format!("Expected {expected:?}, got {item:?}"),
        })
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
    let item = stream
        .next()
        .await
        .ok_or_else(|| FluxionError::InvalidState {
            message: "Expected next timestamped item but stream ended".to_string(),
        })?;

    if item.value == expected {
        Ok(())
    } else {
        Err(FluxionError::InvalidState {
            message: format!("Expected {expected:?}, got {:?}", item.value),
        })
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
        .ok_or_else(|| FluxionError::InvalidState {
            message: "Expected next pair but stream ended".to_string(),
        })?;

    if left.value == expected_left && right.value == expected_right {
        Ok(())
    } else {
        Err(FluxionError::InvalidState {
            message: format!(
                "Expected ({expected_left:?}, {expected_right:?}), got ({:?}, {:?})",
                left.value, right.value
            ),
        })
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
