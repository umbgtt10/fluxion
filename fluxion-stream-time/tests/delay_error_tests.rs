use chrono::Duration;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};

#[tokio::test]
async fn test_delay_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);
    let mut delayed = FluxionStream::new(stream).delay(delay_duration);

    // Act - Send value
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Should arrive after delay
    assert_eq!(unwrap_stream(&mut delayed, 1000).await.unwrap().value, person_alice());

    // Act - Send error - should pass through immediately
    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert - Error should arrive immediately (no delay)
    let error_result = unwrap_stream(&mut delayed, 100).await;
    assert!(matches!(error_result, StreamItem::Error(_)));

    // Act - Send another value
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Should arrive after delay
    assert_eq!(unwrap_stream(&mut delayed, 1000).await.unwrap().value, person_bob());

    Ok(())
}
