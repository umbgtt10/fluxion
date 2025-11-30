// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{DistinctUntilChangedExt, FluxionStream};
use fluxion_test_utils::{helpers::unwrap_stream, test_channel, unwrap_value, Sequenced};

#[tokio::test]
async fn test_filter_ordered_distinct_until_changed() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Composition: filter -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 0) // Filter out negatives
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::with_timestamp(-5, 1))?; // Filtered out
    tx.send(Sequenced::with_timestamp(1, 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 1);

    tx.send(Sequenced::with_timestamp(1, 3))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::with_timestamp(2, 4))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 2);

    tx.send(Sequenced::with_timestamp(-10, 5))?; // Filtered out
    tx.send(Sequenced::with_timestamp(2, 6))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::with_timestamp(3, 7))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 3);

    drop(tx);

    Ok(())
}
