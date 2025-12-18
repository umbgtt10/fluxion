// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    assert_no_element_emitted,
    helpers::unwrap_stream,
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let item = stream_item;
        Sequenced::new(format!(
            "Previous: {:?}, Current: {}",
            item.previous.map(|p| p.value.to_string()),
            &item.current.value
        ))
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        String::from(
            "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
        )
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        String::from(
            "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_to_struct() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, PartialEq, Clone, Ord, PartialOrd, Eq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
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

        Sequenced::new(AgeComparison {
            previous_age,
            current_age,
            age_increased,
        })
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice()))?; // Age 25 again
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        AgeComparison {
            previous_age: Some(25),
            current_age: 35,
            age_increased: true,
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let stream = person_stream
        .ordered_merge(vec![animal_stream])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_str = &item.current.value.to_string();
            let prev_str = item.previous.map(|p| p.value.to_string());
            Sequenced::new(format!("Current: {}, Previous: {:?}", curr_str, prev_str))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    let mut stream = Box::pin(stream);
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value;
    assert!(result.contains("Alice") && result.contains("Previous: None"));

    animal_tx.send(Sequenced::new(animal_dog()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value;
    assert!(result.contains("Dog") && result.contains("Alice"));

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value;
    assert!(result.contains("Bob") && result.contains("Dog"));

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut stream = person_stream
        .combine_latest(vec![animal_stream], |_| true)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_state = item.current.values();
            let count = curr_state.len();
            Sequenced::new(format!("Combined {} streams", count))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined 2 streams"
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined 2 streams"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_filter_age_change() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let item = stream_item;
        let current_age = match &item.current.value {
            TestData::Person(p) => p.age,
            _ => return Sequenced::new(None),
        };
        let previous_age = item.previous.and_then(|prev| match &prev.value {
            TestData::Person(p) => Some(p.age),
            _ => None,
        });

        Sequenced::new(match previous_age {
            Some(prev) if current_age != prev => {
                Some(format!("Age changed from {} to {}", prev, current_age))
            }
            _ => None,
        })
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        None
    ); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        Some(String::from("Age changed from 25 to 30"))
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        Some(String::from("Age changed from 30 to 35"))
    );

    Ok(())
}

#[tokio::test]
async fn test_triple_ordered_merge_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let plant_stream = plant_rx;

    let mut stream = person_stream
        .ordered_merge(vec![animal_stream, plant_stream])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let variant = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            Sequenced::new(variant.to_string())
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    let results = [
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
    ];
    assert!(results.iter().any(|r| r.value == "Person"));
    assert!(results.iter().any(|r| r.value == "Animal"));
    assert!(results.iter().any(|r| r.value == "Plant"));

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered() -> anyhow::Result<()> {
    // Arrange - filter for people only, then map to names
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .map_ordered(|stream_item| {
            let item = stream_item;
            Sequenced::new(match &item.value {
                TestData::Person(p) => format!("Person: {}", p.name),
                _ => unreachable!(),
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Person: Alice"
    );

    tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut stream, 500).await;

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Person: Bob"
    );

    tx.send(Sequenced::new(plant_rose()))?;
    assert_no_element_emitted(&mut stream, 500).await;

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Person: Charlie"
    );

    Ok(())
}
