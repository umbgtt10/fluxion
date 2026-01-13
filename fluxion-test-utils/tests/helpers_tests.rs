use fluxion_core::{FluxionError, StreamItem};
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel, test_channel_with_errors,
    unwrap_stream, unwrap_value,
};

#[tokio::test]
async fn test_assert_no_element_emitted() {
    let (_tx, mut stream) = test_channel::<i32>();

    // This should pass as no elements are sent
    assert_no_element_emitted(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic = "Timeout: No item received within 100 ms"]
async fn test_unwrap_stream_timeout() {
    let (_tx, mut stream) = test_channel::<i32>();

    // This should panic due to timeout
    unwrap_stream(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic = "Expected StreamItem but stream ended"]
async fn test_unwrap_stream_empty() {
    let (tx, mut stream) = test_channel::<i32>();

    // Close the stream immediately
    drop(tx);

    // This should panic because the stream ends
    unwrap_stream(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Expected Value but got Error: Stream processing error: injected error"]
async fn test_unwrap_stream_error_injected() {
    let (tx, mut stream) = test_channel_with_errors::<i32>();

    tx.try_send(StreamItem::Error(FluxionError::stream_error(
        "injected error",
    )))
    .unwrap();

    // This should panic because unwrap_value expects a Value
    let item = unwrap_stream(&mut stream, 500).await;
    unwrap_value(Some(item));
}

#[tokio::test]
async fn test_assert_stream_ended_success() {
    let (tx, mut stream) = test_channel::<i32>();

    // Close the stream
    drop(tx);

    // This should pass because the stream has ended
    assert_stream_ended(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Expected stream to end but it returned a value"]
async fn test_assert_stream_ended_returns_value() {
    let (tx, mut stream) = test_channel::<i32>();

    // Send a value
    tx.try_send(42).unwrap();

    // This should panic because the stream returns a value
    assert_stream_ended(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Timeout: Stream did not end within 100 ms"]
async fn test_assert_stream_ended_timeout() {
    let (_tx, mut stream) = test_channel::<i32>();

    // Stream is open but no values sent - will timeout
    assert_stream_ended(&mut stream, 100).await;
}
