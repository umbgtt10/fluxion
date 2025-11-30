// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_rx::FluxionStream;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::Sequenced;

#[tokio::test]
async fn test_skip_items_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream)
        .skip_items(2) // Skip first 2
        .filter_ordered(|x| *x > 15); // Then filter

    // Act
    tx.send(Sequenced::new(5))?; // Skipped
    tx.send(Sequenced::new(10))?; // Skipped
    tx.send(Sequenced::new(12))?; // Not skipped, but filtered (12 <= 15)
    tx.send(Sequenced::new(20))?; // Emitted
    tx.send(Sequenced::new(30))?; // Emitted

    // Assert - Only values > 15 after skip
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item1.into_inner(), 20);

    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.into_inner(), 30);

    Ok(())
}

#[tokio::test]
async fn test_start_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(1)),
        StreamItem::Value(Sequenced::new(2)),
    ];

    let mut result = FluxionStream::new(stream)
        .start_with(initial)
        .combine_with_previous();

    // Act
    tx.send(Sequenced::new(3))?;
    tx.send(Sequenced::new(4))?;

    // Assert - First has no previous
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(item1.previous.is_none());
    assert_eq!(item1.current.into_inner(), 1);

    // Second has previous = 1
    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.previous.unwrap().into_inner(), 1);
    assert_eq!(item2.current.into_inner(), 2);

    // Third has previous = 2
    let item3 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item3.previous.unwrap().into_inner(), 2);
    assert_eq!(item3.current.into_inner(), 3);

    // Fourth has previous = 3
    let item4 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item4.previous.unwrap().into_inner(), 3);
    assert_eq!(item4.current.into_inner(), 4);

    Ok(())
}

#[tokio::test]
async fn test_complex_chain_with_all_three_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(0)),
        StreamItem::Value(Sequenced::new(1)),
    ];

    // Start with 2 initial values, skip 1, take 4, then map
    let mut result = FluxionStream::new(stream)
        .start_with(initial)
        .skip_items(1) // Skip the 0
        .take_items(4) // Take next 4: [1, 2, 3, 4]
        .map_ordered(|item| item.value * 10);

    // Act
    tx.send(Sequenced::new(2))?;
    tx.send(Sequenced::new(3))?;
    tx.send(Sequenced::new(4))?;
    tx.send(Sequenced::new(5))?; // Should not be emitted (take limit)

    // Assert - [1, 2, 3, 4] * 10 = [10, 20, 30, 40]
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(10)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(20)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(30)));

    let item4 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item4, StreamItem::Value(40)));

    fluxion_test_utils::helpers::assert_stream_ended(&mut result, 100).await;

    Ok(())
}
