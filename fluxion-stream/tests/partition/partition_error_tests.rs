// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::PartitionExt;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, TestData,
};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{assert_stream_ended, test_channel_with_errors};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value};

#[tokio::test]
async fn test_partition_propagates_error_to_both_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act - send an error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert - both streams receive the error
    assert!(matches!(
        unwrap_stream(&mut persons, 500).await,
        StreamItem::Error(_)
    ));

    assert!(matches!(
        unwrap_stream(&mut non_persons, 500).await,
        StreamItem::Error(_)
    ));

    // Both streams should complete after error
    assert_stream_ended(&mut persons, 500).await;
    assert_stream_ended(&mut non_persons, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_error_after_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act - send values then error
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "error after values",
    )))?;

    // Assert - values arrive before error
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_bob()
    );

    // Error should arrive on both streams
    assert!(matches!(
        unwrap_stream(&mut persons, 500).await,
        StreamItem::Error(_)
    ));

    assert!(matches!(
        unwrap_stream(&mut non_persons, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_partition_error_message_preserved() -> anyhow::Result<()> {
    // Arrange - partition by type
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act
    let error_msg = "custom partition error";
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(error_msg)))?;

    // Assert - error message is preserved in both streams
    let result1 = unwrap_stream(&mut persons, 500).await;
    if let StreamItem::Error(e) = result1 {
        assert!(e.to_string().contains(error_msg));
    } else {
        panic!("Expected error on persons stream");
    }

    let result2 = unwrap_stream(&mut non_persons, 500).await;
    if let StreamItem::Error(e) = result2 {
        assert!(e.to_string().contains(error_msg));
    } else {
        panic!("Expected error on non_persons stream");
    }

    Ok(())
}

#[tokio::test]
async fn test_partition_mixed_values_and_error() -> anyhow::Result<()> {
    // Arrange - partition by type (Person vs others)
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act - send mixed partition values then error
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?; // person
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?; // non-person
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?; // person
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "mid-stream error",
    )))?;
    // These won't be received due to error termination
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_spider())))?;

    // Assert - values arrive in correct streams
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut non_persons, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_bob()
    );

    // Error arrives on both streams
    assert!(matches!(
        unwrap_stream(&mut persons, 500).await,
        StreamItem::Error(_)
    ));

    assert!(matches!(
        unwrap_stream(&mut non_persons, 500).await,
        StreamItem::Error(_)
    ));

    // Streams should be ended
    assert_stream_ended(&mut persons, 500).await;
    assert_stream_ended(&mut non_persons, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_error_terminates_routing() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act - send error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "termination error",
    )))?;

    // Try to send more (these should be ignored by subjects since they're closed)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;

    // Assert - only error is received
    assert!(matches!(
        unwrap_stream(&mut persons, 500).await,
        StreamItem::Error(_)
    ));

    assert!(matches!(
        unwrap_stream(&mut non_persons, 500).await,
        StreamItem::Error(_)
    ));

    // Both ended after error
    assert_stream_ended(&mut persons, 500).await;
    assert_stream_ended(&mut non_persons, 500).await;

    Ok(())
}
