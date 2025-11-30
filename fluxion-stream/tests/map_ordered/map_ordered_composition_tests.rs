// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            StreamItem::Value(format!(
                "Previous: {:?}, Current: {}",
                item.previous.map(|p| p.value.to_string()),
                &item.current.value
            ))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
        )
    );

    tx.send(Sequenced::new(person_charlie()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_to_struct() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });
            let age_increased = previous_age.is_some_and(|prev| current_age > prev);

            StreamItem::Value(AgeComparison {
                previous_age,
                current_age,
                age_increased,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice()))?; // Age 25 again
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(25),
            current_age: 35,
            age_increased: true,
        }
    );

    Ok(())
}
