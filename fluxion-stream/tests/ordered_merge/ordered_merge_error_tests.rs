// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::OrderedStreamExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::TestData,
    test_data::{
        animal_bird, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_dave, plant_oak, plant_rose,
    },
};

#[tokio::test]
async fn test_ordered_merge_propagates_error_from_first_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Error from stream 1",
    )))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == person_alice())
    );

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == animal_dog())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_propagates_error_from_second_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Error from stream 2",
    )))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == person_bob())
    );

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == person_charlie())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_multiple_errors_from_different_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        plant_rose(),
        1,
    )))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        4,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == plant_rose())
    );

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == animal_spider())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_dave(),
        1,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == person_dave())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_before_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_bird(),
        1,
    )))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == animal_bird())
    );

    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    drop(tx1);
    drop(tx2);

    // Assert
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_consecutive_errors_same_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(plant_oak(), 1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(
        matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == plant_oak())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_error_ordering_by_timestamp() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    // Assert
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 1));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 2));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 3));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 4));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_three_streams_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx3, stream3) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2, stream3]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx3.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 1));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 2));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 3));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_only_errors_no_values() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_continues_after_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 1));
    assert!(matches!(
        unwrap_stream(&mut merged, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 2));
    assert!(matches!(&unwrap_stream(&mut merged, 100).await, StreamItem::Value(v) if v.value == 3));

    Ok(())
}
