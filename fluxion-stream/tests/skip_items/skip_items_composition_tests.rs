// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, person_diane, TestData,
};
use fluxion_test_utils::{test_channel, Sequenced};

#[tokio::test]
async fn test_map_ordered_skip_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = FluxionStream::new(stream)
        .map_ordered(|item| Sequenced::new(item.into_inner())) // Preserve Sequenced wrapper
        .skip_items(2);

    // Act
    tx.send(Sequenced::new(person_alice()))?; // age=25, Skipped
    tx.send(Sequenced::new(person_dave()))?; // age=28, Skipped
    tx.send(Sequenced::new(person_bob()))?; // age=30, Emitted
    tx.send(Sequenced::new(person_charlie()))?; // age=35, Emitted
    tx.send(Sequenced::new(person_diane()))?; // age=40, Emitted

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
