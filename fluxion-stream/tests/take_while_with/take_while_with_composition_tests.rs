// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

static LATEST_FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (latest_filter_tx, latest_filter_rx) = test_channel::<Sequenced<TestData>>();
    let (while_filter_tx, while_filter_rx) = test_channel::<Sequenced<bool>>();

    let source_stream = source_rx;
    let latest_filter_stream = latest_filter_rx;
    let while_filter_stream = while_filter_rx;

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(latest_filter_stream, LATEST_FILTER)
        .take_while_with(while_filter_stream, |f| *f);

    // Act & Assert
    while_filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    latest_filter_tx
        .send(Sequenced::new(person_alice()))
        .unwrap();
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    latest_filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_bob());

    while_filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    latest_filter_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}
