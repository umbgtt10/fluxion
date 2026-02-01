// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{MapOrderedExt, OrderedStreamExt, SkipItemsExt};
use fluxion_test_utils::helpers::{test_channel, unwrap_stream, unwrap_value};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, person_diane, TestData,
};

#[tokio::test]
async fn test_map_ordered_skip_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = stream
        .map_ordered(|item| Sequenced::new(item.into_inner()))
        .skip_items(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

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

    let mut stream = s1_rx.ordered_merge(vec![s2_rx]).skip_items(3);

    // Act
    s1_tx.unbounded_send(Sequenced::new(person_alice()))?;
    s2_tx.unbounded_send(Sequenced::new(person_bob()))?;
    s1_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    s2_tx.unbounded_send(Sequenced::new(person_dave()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_dave()
    );

    // Act
    s1_tx.unbounded_send(Sequenced::new(person_diane()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_diane()
    );

    Ok(())
}
