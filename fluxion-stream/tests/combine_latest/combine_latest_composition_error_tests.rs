// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{CombineLatestExt, CombineWithPreviousExt, FilterOrderedExt, MapOrderedExt};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_combine_latest_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = stream1
        .map_ordered(|x| {
            // Identity map
            x
        })
        .combine_latest(vec![stream2], |_| true);

    // Act: Send initial values
    tx1.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // First emission
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error from primary stream (which is mapped)
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_then_combine_latest_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = stream1
        .filter_ordered(|_| true)
        .combine_latest(vec![stream2], |_| true);

    // Act: Send initial values
    tx1.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // First emission
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error from primary stream (which is filtered)
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_then_combine_latest_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = stream1
        .combine_with_previous()
        .combine_latest(vec![stream2.combine_with_previous()], |_| true);

    // Act: Send initial values
    tx1.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error from primary stream
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Previous error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}
