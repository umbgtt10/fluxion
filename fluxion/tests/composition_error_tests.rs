// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0
//! Error propagation tests for composed stream operations.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{
    CombineLatestExt, DistinctUntilChangedByExt, DistinctUntilChangedExt, EmitWhenExt,
    FluxionStream, TakeLatestWhenExt,
};
use fluxion_test_utils::{test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_error_propagation_through_multiple_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain multiple operators
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x > 1) // Filter out first item
        .combine_with_previous()
        .map_ordered(|x| Sequenced::new(x.current.value * 10));

    // Act & Assert Send value (filtered out)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Send value (passes)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 20
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 40
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 50
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_error_in_long_operator_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Long chain: filter -> combine_with_previous -> map
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 10)
        .combine_with_previous()
        .map_ordered(|x| Sequenced::new(x.current.value + 5));

    // Act & Assert Send value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 15
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 35
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(40, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 45
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_multiple_errors_through_composition() -> anyhow::Result<()> {
    // ASrrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    // Combine then transform
    let mut result = stream1
        .combine_latest(vec![stream2], |_| true)
        .combine_with_previous()
        .map_ordered(|x| {
            let state = &x.current;
            Sequenced::new(format!("Combined: {:?}", state.values()))
        });

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(10, 4)))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref s) if s.value.contains("Combined"))
    );

    // Send error
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref s) if s.value.contains("Combined"))
    );

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_in_composed_streams() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Complex composition
    let mut result = source_stream
        .take_latest_when(trigger_stream, |_| true)
        .combine_with_previous()
        .map_ordered(|x| Sequenced::new(x.current.value));

    // Send source values
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;

    // Send error
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(15, 3)))?;
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(100, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 15
    ));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_error_with_emit_when_composition() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // emit_when + map composition
    let mut result = source_stream
        .emit_when(filter_stream, |state| state.values()[0] > state.values()[1])
        .combine_with_previous()
        .map_ordered(|x| Sequenced::new(x.current.value * 2));

    // Arrange & Act Send filter value first
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Send value that passes
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(20, 4)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(25, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 50
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_error_propagation_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Composition: distinct_until_changed -> filter -> combine_with_previous
    let mut result = FluxionStream::new(stream)
        .distinct_until_changed()
        .filter_ordered(|x| *x >= 0)
        .combine_with_previous();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    let combined = unwrap_stream(&mut result, 100).await;
    if let StreamItem::Value(val) = combined {
        assert_eq!(val.current.value, 1);
        assert_eq!(val.previous, None);
    } else {
        panic!("Expected Value");
    }

    // Send duplicate (filtered by distinct)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue - state should be preserved
    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 4)))?;
    let combined = unwrap_stream(&mut result, 100).await;
    if let StreamItem::Value(val) = combined {
        assert_eq!(val.current.value, 2);
        assert_eq!(val.previous.as_ref().unwrap().value, 1);
    } else {
        panic!("Expected Value");
    }

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_multiple_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Composition: map -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let doubled = s.value * 2;
            Sequenced::new(doubled)
        })
        .distinct_until_changed();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Send duplicate value (5*2=10, same as before)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;

    // Send another error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Send different value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(7, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_error_propagation_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<String>>();

    // Composition: map -> distinct_until_changed_by (case-insensitive) -> filter
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let trimmed = s.value.trim().to_string();
            Sequenced::new(trimmed)
        })
        .distinct_until_changed_by(|a, b| a.to_lowercase() == b.to_lowercase())
        .filter_ordered(|s| !s.is_empty());

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        "Hello".to_string(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Same (case-insensitive) - filtered by distinct_by
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        "HELLO".to_string(),
        2,
    )))?;

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue - state should be preserved
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        "hello".to_string(),
        4,
    )))?; // Still same case-insensitive

    // Different value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        "World".to_string(),
        5,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

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

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Same parity - filtered by distinct_by
    tx.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Another error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Different parity - emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 5)))?; // Even
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_propagation_with_map() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let accumulator = |sum: &mut i32, value: &i32| {
        *sum += value;
        *sum
    };

    let mut result = FluxionStream::new(stream)
        .scan_ordered(0, accumulator)
        .map_ordered(|sum: Sequenced<i32>| Sequenced::new(sum.into_inner() * 2));

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 20 // sum=10, doubled=20
    ));

    // Error doesn't affect accumulator state
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Accumulator continues from previous state
    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 30 // sum=15, doubled=30
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
    tx.send(StreamItem::Value(Sequenced::with_timestamp(200, 2)))?; // count=2, emitted
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    // Error propagates through
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Count continues, filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?; // count=3, filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?; // count=4, emitted
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 4
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_chained_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain two scans with errors in between
    let mut result = FluxionStream::new(stream)
        .scan_ordered::<Sequenced<i32>, _, _>(0, |sum: &mut i32, value: &i32| {
            *sum += value;
            *sum
        })
        .scan_ordered(0, |count: &mut i32, _sum: &i32| {
            *count += 1;
            *count
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?; // sum=10, count=1
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    // Error between scans
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Both accumulators preserve state
    tx.send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?; // sum=30, count=2
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(30, 4)))?; // sum=60, count=3
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 3
    ));

    drop(tx);

    Ok(())
}
