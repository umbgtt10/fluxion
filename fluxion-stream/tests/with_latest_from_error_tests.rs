// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `with_latest_from` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::WithLatestFromExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_with_latest_from_propagates_primary_error() -> anyhow::Result<()> {
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream.with_latest_from(secondary_stream, |_| true);

    // Send secondary first (required for with_latest_from)
    secondary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();

    // Send primary value
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Send error in primary
    primary_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Primary error",
        )))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(
        matches!(item2, StreamItem::Error(_)),
        "Should propagate error from primary stream"
    );

    // Continue with more values
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(3, 4)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(_)));

    drop(primary_tx);
    drop(secondary_tx);
    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_propagates_secondary_error() -> anyhow::Result<()> {
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream.with_latest_from(secondary_stream, |_| true);

    // Send secondary value first
    secondary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();

    // Send primary value
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Send error in secondary
    secondary_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Secondary error",
        )))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(
        matches!(item2, StreamItem::Error(_)),
        "Should propagate error from secondary"
    );

    // Update secondary with new value
    secondary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(30, 5)))
        .unwrap();

    // Send primary value
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(2, 6)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(_)));

    drop(primary_tx);
    drop(secondary_tx);
    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_error_before_secondary_ready() -> anyhow::Result<()> {
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream.with_latest_from(secondary_stream, |_| true);

    // Send error in primary before secondary has value
    primary_tx
        .send(StreamItem::Error(FluxionError::stream_error("Early error")))
        .unwrap();

    let item = result.next().await.unwrap();
    assert!(matches!(item, StreamItem::Error(_)));

    drop(primary_tx);
    drop(secondary_tx);
    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_selector_continues_after_error() -> anyhow::Result<()> {
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Custom selector - pass through combined state
    let mut result = primary_stream.with_latest_from(secondary_stream, |combined| combined.clone());

    // Send secondary first
    secondary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Send primary value
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Send error in primary
    primary_tx
        .send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Update secondary
    secondary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(200, 4)))
        .unwrap();

    // Send primary value
    primary_tx
        .send(StreamItem::Value(Sequenced::with_sequence(3, 5)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(_)));

    drop(primary_tx);
    drop(secondary_tx);
    Ok(())
}
