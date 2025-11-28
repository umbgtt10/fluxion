// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_unordered::CombineLatestExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream, unwrap_value},
    test_channel,
};

static FILTER: fn(&Vec<i32>) -> bool = |_| true;

#[tokio::test]
async fn test_unordered_combine_latest_basic() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<i32>();
    let (tx2, stream2) = test_channel::<i32>();

    let mut result = stream1.combine_latest(vec![stream2], FILTER);

    // Act - send first values
    tx1.send(10)?;
    tx2.send(20)?;

    // Assert - first emission after both streams have values
    let item = unwrap_stream(&mut result, 500).await;
    let values = unwrap_value(Some(item));
    assert_eq!(values, vec![10, 20]);

    // Act - update first stream
    tx1.send(11)?;

    // Assert - new emission with updated first stream
    let item = unwrap_stream(&mut result, 500).await;
    let values = unwrap_value(Some(item));
    assert_eq!(values, vec![11, 20]);

    Ok(())
}

#[tokio::test]
async fn test_unordered_combine_latest_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<i32>();
    let (tx2, stream2) = test_channel::<i32>();

    let mut result = stream1.combine_latest(vec![stream2], FILTER);

    // Act
    drop(tx1);
    drop(tx2);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_unordered_combine_latest_three_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<i32>();
    let (tx2, stream2) = test_channel::<i32>();
    let (tx3, stream3) = test_channel::<i32>();

    let mut result = stream1.combine_latest(vec![stream2, stream3], FILTER);

    // Act - initialize all streams
    tx1.send(100)?;
    tx2.send(200)?;
    tx3.send(300)?;

    // Assert
    let item = unwrap_stream(&mut result, 500).await;
    let values = unwrap_value(Some(item));
    assert_eq!(values, vec![100, 200, 300]);

    // Act - update middle stream
    tx2.send(201)?;

    // Assert
    let item = unwrap_stream(&mut result, 500).await;
    let values = unwrap_value(Some(item));
    assert_eq!(values, vec![100, 201, 300]);

    Ok(())
}

#[tokio::test]
async fn test_unordered_combine_latest_with_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<i32>();
    let (tx2, stream2) = test_channel::<i32>();

    // Filter: only emit when sum > 25
    let filter = |values: &Vec<i32>| values[0] + values[1] > 25;

    let mut result = stream1.combine_latest(vec![stream2], filter);

    // Act - send values with sum = 15 (filtered out)
    tx1.send(5)?;
    tx2.send(10)?;

    // Assert - no emission (filtered)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act - update to sum = 30 (passes filter)
    tx1.send(20)?;

    // Assert - emission received
    let item = unwrap_stream(&mut result, 500).await;
    let values = unwrap_value(Some(item));
    assert_eq!(values, vec![20, 10]);
    assert_eq!(values[0] + values[1], 30);

    Ok(())
}
