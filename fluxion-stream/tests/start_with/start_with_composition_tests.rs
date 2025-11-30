// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::{FluxionStream, WithPrevious};
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, TestData,
};
use fluxion_test_utils::{test_channel, Sequenced};

#[tokio::test]
async fn test_combine_with_previous_start_with() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let initial = vec![
        StreamItem::Value(WithPrevious::new(None, Sequenced::new(person_alice()))),
        StreamItem::Value(WithPrevious::new(
            Some(Sequenced::new(person_alice())),
            Sequenced::new(person_bob()),
        )),
    ];

    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .start_with(initial);

    // Act
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?;

    // Assert - First two from start_with (prepended)
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(item1.previous.is_none() && item1.current.into_inner() == person_alice());

    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(
        item2.previous.clone().map(|p| p.into_inner()) == Some(person_alice())
            && item2.current.into_inner() == person_bob()
    );

    // Third from stream - charlie (first emission has previous = None since it''s first from actual stream)
    let item3 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(item3.previous.is_none() && item3.current.into_inner() == person_charlie());

    // Fourth from stream - dave (has previous = charlie)
    let item4 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(
        item4.previous.clone().map(|p| p.into_inner()) == Some(person_charlie())
            && item4.current.into_inner() == person_dave()
    );

    Ok(())
}
