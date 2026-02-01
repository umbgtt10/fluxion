// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{CombinedState, WithLatestFromExt};
use fluxion_test_utils::helpers::{test_channel_with_errors, unwrap_stream};
use fluxion_test_utils::test_data::{animal_cat, animal_dog, person_alice, person_bob, TestData};
use fluxion_test_utils::test_wrapper::TestWrapper;
use fluxion_test_utils::{helpers::assert_no_element_emitted, sequenced::Sequenced};

#[tokio::test]
async fn test_with_latest_from_propagates_primary_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<TestData, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_cat(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<TestWrapper<bool>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value() == &true
    ));

    // Act
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Primary error",
    )))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)),
        "Should propagate error from primary stream"
    );

    // Act
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<TestWrapper<bool>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value() == &true
    ));

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_propagates_secondary_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<TestData, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_cat(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<TestWrapper<bool>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value() == &true
    ));

    // Act
    secondary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Secondary error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        5,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        6,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<TestWrapper<bool>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value() == &true
    ));

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_error_before_secondary_ready() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (_, secondary_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<TestData, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_selector_continues_after_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = primary_stream.with_latest_from(secondary_stream, |combined| combined.clone());

    // Act
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_cat(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<CombinedState<TestData, u64>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.values().len() == 2
    ));

    // Act
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        4,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        5,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<CombinedState<TestData, u64>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.values().len() == 2
    ));

    drop(primary_tx);
    drop(secondary_tx);

    Ok(())
}
