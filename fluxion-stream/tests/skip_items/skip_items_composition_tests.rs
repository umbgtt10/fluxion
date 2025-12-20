// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{MapOrderedExt, OrderedStreamExt, SkipItemsExt};
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, person_diane, TestData,
};
use fluxion_test_utils::{test_channel, unwrap_value, Sequenced};

#[tokio::test]
async fn test_map_ordered_skip_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = stream
        .map_ordered(|item| Sequenced::new(item.into_inner())) // Preserve Sequenced wrapper
        .skip_items(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age=25, Skipped
    tx.unbounded_send(Sequenced::new(person_dave()))?; // age=28, Skipped
    tx.unbounded_send(Sequenced::new(person_bob()))?; // age=30, Emitted
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age=35, Emitted
    tx.unbounded_send(Sequenced::new(person_diane()))?; // age=40, Emitted

    // Assert - All items after skip
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_skip_items() -> anyhow::Result<()> {
    // Arrange
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    // Merge streams then skip 3 items
    let mut stream = s1_rx.ordered_merge(vec![s2_rx]).skip_items(3);

    // Act & Assert
    // 1. Stream 1 emits (Skipped #1)
    s1_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // 2. Stream 2 emits (Skipped #2)
    s2_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // 3. Stream 1 emits (Skipped #3)
    s1_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // 4. Stream 2 emits (Should be emitted)
    s2_tx.unbounded_send(Sequenced::new(person_dave()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_dave()
    );

    // 5. Stream 1 emits (Should be emitted)
    s1_tx.unbounded_send(Sequenced::new(person_diane()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_diane()
    );

    Ok(())
}
