// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_error_propagation_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = FluxionStream::new(stream)
        .distinct_until_changed()
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 10,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    // 1. Send Alice (Age 25) -> Should be emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_alice() && val.previous.is_none()
    ));

    // 2. Send Alice again -> Should be filtered by distinct_until_changed
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 3. Send Error -> Should be propagated
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // 4. Send Bob (Age 30) -> Should be emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_bob() && val.previous.as_ref().map(|p| &p.value) == Some(&person_alice())
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_then_combine_with_previous_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = FluxionStream::new(stream)
        .map_ordered(|x| x) // Identity map
        .combine_with_previous();

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_then_combine_with_previous_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = FluxionStream::new(stream)
        .filter_ordered(|_| true) // Pass everything
        .combine_with_previous();

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // First emission
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Should propagate error
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}
