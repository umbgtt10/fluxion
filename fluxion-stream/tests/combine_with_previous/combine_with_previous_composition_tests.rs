// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_value, Sequenced,
};

static FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(filter_stream, FILTER)
        .combine_with_previous();

    // Act & Assert - send source first, then filter triggers emission
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;

    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(
        element.previous.is_none(),
        "First emission should have no previous"
    );
    assert_eq!(&element.current.value, &person_alice());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_alice());
    assert_eq!(&element.current.value, &person_bob());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_charlie()))?;
    filter_tx.send(Sequenced::new(person_charlie()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_bob());
    assert_eq!(&element.current.value, &person_charlie());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_dave()))?;
    filter_tx.send(Sequenced::new(person_dave()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}
