// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `ordered_merge` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::OrderedStreamExt;
use fluxion_test_utils::{
    assert_stream_ended, test_channel_with_errors,
    test_data::{
        animal_bird, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_dave, plant_oak, plant_rose,
    },
    unwrap_stream, Sequenced, TestData,
};

#[tokio::test]
async fn test_ordered_merge_propagates_error_from_first_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send values and error from first stream
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error(
        "Error from stream 1",
    )))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;

    // Assert: First value should be emitted
    let item1 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.into_inner(), person_alice());
    }

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    // Second value should still be emitted
    let item2 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        assert_eq!(v.into_inner(), animal_dog());
    }

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_propagates_error_from_second_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        1,
    )))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error(
        "Error from stream 2",
    )))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        3,
    )))?;

    // Assert: Error has priority (Error < Value), so it's emitted first
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    // Then values in timestamp order
    let item1 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.into_inner(), person_bob());
    }

    let item2 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        assert_eq!(v.into_inner(), person_charlie());
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_multiple_errors_from_different_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Interleave values and errors
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_rose(),
        1,
    )))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        4,
    )))?;

    // Assert: Error1 (from stream2) has priority over Value(rose,1), then Value(rose,1),
    // then Error2 has priority over Value(spider,4), then Value(spider,4)
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    let item1 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.into_inner(), plant_rose());
    }

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    let item2 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item2 {
        assert_eq!(v.into_inner(), animal_spider());
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send error before any values
    tx1.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        person_dave(),
        1,
    )))?;

    // Assert: Error should be propagated immediately
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    // Stream should continue with values
    let item = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item {
        assert_eq!(v.into_inner(), person_dave());
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_before_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_bird(),
        1,
    )))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item {
        assert_eq!(v.into_inner(), animal_bird());
    }

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    // Drop transmitters to end streams
    drop(tx1);
    drop(tx2);

    // Assert: Stream should end cleanly after error
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_consecutive_errors_same_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send multiple consecutive errors from same stream
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(plant_oak(), 1)))?;

    // Assert: Both errors should propagate
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_ordering_by_timestamp() -> anyhow::Result<()> {
    // Arrange: Test that errors are emitted in timestamp order
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send values and errors with explicit timestamps
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Error has priority (Error < Value), so it's emitted before buffered value with ts=3
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    tx1.send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    // Assert: Values with ts=1,2 emitted first (already in buffer), then error (priority), then remaining values
    let item1 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));

    let item2 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));

    let item3 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item3, StreamItem::Error(_)));

    let item4 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item4, StreamItem::Value(_)));

    let item5 = unwrap_stream(&mut merged, 100).await;
    assert!(matches!(item5, StreamItem::Value(_)));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_three_streams_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx3, stream3) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2, stream3]);

    // Act: Send values and errors from all three streams
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx3.send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert: Error1, Value(1,1), Error2, Value(2,2), Value(3,3)
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_only_errors_no_values() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send only errors, no values
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert: Both errors should be propagated
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx1);
    drop(tx2);

    // Stream should end after errors
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_continues_after_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act: Send pattern of value, error, error, value
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert: Error2, Value(1,1), Error1, Value(2,2), Value(3,3)
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}
