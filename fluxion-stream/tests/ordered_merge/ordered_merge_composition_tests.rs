// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_take_latest_when_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    static LATEST_FILTER_LOCAL: fn(&TestData) -> bool = |_| true;

    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let filter_stream = filter_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(FluxionStream::new(filter_stream), LATEST_FILTER_LOCAL)
        .ordered_merge(vec![FluxionStream::new(FluxionStream::new(animal_stream))]);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    let result1 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));

    let values: Vec<_> = vec![&result1.value, &result2.value];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&result.value, &person_bob());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_charlie()))?;
    // After filter stream closes, no more emissions should occur
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}
