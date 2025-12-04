// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_multiple_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Composition: distinct_until_changed_by (parity) -> map -> filter
    let mut result = FluxionStream::new(stream)
        .distinct_until_changed_by(|a, b| a % 2 == b % 2)
        .map_ordered(|s| {
            let doubled = s.value * 2;
            Sequenced::new(doubled)
        })
        .filter_ordered(|x| *x > 0);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?; // Odd
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 5)))?; // Even
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_propagation_with_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let accumulator = |count: &mut i32, _value: &i32| {
        *count += 1;
        *count
    };

    let mut result = FluxionStream::new(stream)
        .scan_ordered(0, accumulator)
        .filter_ordered(|count| count % 2 == 0); // Only even counts

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?; // count=1, filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(StreamItem::Value(Sequenced::with_timestamp(200, 2)))?; // count=2, emitted
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?; // count=3, filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?; // count=4, emitted
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 4
    ));

    drop(tx);

    Ok(())
}
