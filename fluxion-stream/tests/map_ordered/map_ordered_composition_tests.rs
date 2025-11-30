// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::{CombinedState, FluxionStream, WithPrevious};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};
use futures::StreamExt;

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

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

#[tokio::test]
async fn test_combine_with_previous_map_ordered_age_difference() -> anyhow::Result<()> {
    // Arrange
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

            StreamItem::Value(match previous_age {
                Some(prev) => current_age as i32 - prev as i32,
                None => 0,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        0
    ); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        5
    ); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave()))?; // Age 28
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        -2
    ); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        7
    ); // 35 - 28 = 7
    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_str = &item.current.value.to_string();
            let prev_str = item.previous.map(|p| p.value.to_string());
            StreamItem::Value(format!("Current: {}, Previous: {:?}", curr_str, prev_str))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    let mut stream = Box::pin(stream);
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("Previous: None"));

    animal_tx.send(Sequenced::new(animal_dog()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Dog"));
    assert!(result.contains("Alice"));

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Bob"));
    assert!(result.contains("Dog"));

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_combine_with_previous_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

    let mut stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_state = item.current.values();
            let count = curr_state.len();
            StreamItem::Value(format!("Combined {} streams", count))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        "Combined 2 streams"
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        "Combined 2 streams"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_filter_age_change() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => return StreamItem::Value(None),
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            StreamItem::Value(match previous_age {
                Some(prev) if current_age != prev => {
                    Some(format!("Age changed from {} to {}", prev, current_age))
                }
                _ => None,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        None
    ); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        Some(String::from("Age changed from 25 to 30"))
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        Some(String::from("Age changed from 30 to 35"))
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_emit_when_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let threshold_stream = threshold_rx;

    let threshold_mapped =
        threshold_stream.map(|seq| StreamItem::Value(WithPrevious::new(None, seq.unwrap())));

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let current_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let threshold_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        current_age >= threshold_age
    };

    let mut stream = FluxionStream::new(source_stream)
        .combine_with_previous()
        .emit_when(threshold_mapped, filter_fn)
        .map_ordered(|stream_item| {
            let item = stream_item;
            StreamItem::Value(format!("Passed filter: {}", &item.current.value))
        });

    // Act & Assert
    threshold_tx.send(Sequenced::new(person_bob()))?; // Threshold 30
    source_tx.send(Sequenced::new(person_alice()))?; // 25 - below threshold
    assert_no_element_emitted(&mut stream, 100).await;

    source_tx.send(Sequenced::new(person_charlie()))?; // 35 - above threshold
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Charlie"));
    assert!(result.contains("Passed filter"));

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

    let mut stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![
            FluxionStream::new(animal_stream),
            FluxionStream::new(plant_stream),
        ])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let variant = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            StreamItem::Value(variant.to_string())
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    let result1 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result3 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

    let results = [result1, result2, result3];
    assert!(results.contains(&StreamItem::Value(String::from("Person"))));
    assert!(results.contains(&StreamItem::Value(String::from("Animal"))));
    assert!(results.contains(&StreamItem::Value(String::from("Plant"))));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_previous_and_map_ordered_name_change() -> anyhow::Result<()> {
    // Arrange - track when name changes between consecutive items
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![FluxionStream::new(s2_rx)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let current_binding = item.current;
            let current_name = match &current_binding.value {
                TestData::Person(p) => p.name.clone(),
                _ => "Unknown".to_string(),
            };
            let prev_name = item.previous.as_ref().map(|p| match &p.value {
                TestData::Person(person) => person.name.clone(),
                _ => "Unknown".to_string(),
            });

            StreamItem::Value(match prev_name {
                Some(prev) if prev != current_name => {
                    format!("Name changed from {} to {}", prev, current_name)
                }
                Some(_) => format!("Same name: {}", current_name),
                None => format!("First entry: {}", current_name),
            })
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "First entry: Alice"
    );

    s2_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Same name: Alice"
    );

    s1_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Name changed from Alice to Bob"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_previous_map_ordered_type_count() -> anyhow::Result<()> {
    // Arrange - count different types across combined streams
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let state_binding = item.current;
            let inner = state_binding.clone().into_inner();
            let state = inner.values();
            let person_count = state
                .iter()
                .filter(|d| matches!(d, TestData::Person(_)))
                .count();
            let animal_count = state
                .iter()
                .filter(|d| matches!(d, TestData::Animal(_)))
                .count();
            StreamItem::Value(format!(
                "Persons: {}, Animals: {}",
                person_count, animal_count
            ))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Persons: 1, Animals: 1"
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Persons: 1, Animals: 1"
    );

    Ok(())
}
