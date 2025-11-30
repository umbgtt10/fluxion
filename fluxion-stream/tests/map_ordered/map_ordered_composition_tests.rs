// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, StreamItem, Timestamped};
use fluxion_stream::{CombinedState, FluxionStream, MergedStream, WithPrevious};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    test_wrapper::TestWrapper,
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
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        String::from(
            "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
        )
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
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

    let results = [
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
    ];
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

#[tokio::test]
async fn test_double_ordered_merge_map_ordered() -> anyhow::Result<()> {
    // Arrange - merge two pairs of streams, then merge results
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let type_name = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            let count = if item.previous.is_some() { 2 } else { 1 };
            StreamItem::Value(format!("{} (item #{})", type_name, count))
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person (item #1)"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal (item #2)"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant (item #2)"
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_ordered_data_extraction() -> anyhow::Result<()> {
    // Arrange - extract specific fields from merged data
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.current.value {
                TestData::Person(p) => format!("Person: {}, Age: {}", p.name, p.age),
                TestData::Animal(a) => format!("Animal: {}, Legs: {}", a.name, a.legs),
                TestData::Plant(p) => format!("Plant: {}, Height: {}", p.species, p.height),
            })
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice, Age: 25"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal: Dog, Legs: 4"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant: Rose, Height: 15"
    );
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered() -> anyhow::Result<()> {
    // Arrange - filter for people only, then map to names
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.value {
                TestData::Person(p) => format!("Person: {}", p.name),
                _ => unreachable!(),
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice"
    );

    tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Bob"
    );

    tx.send(Sequenced::new(plant_rose()))?; // Filtered out
    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Charlie"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - complex pipeline: filter -> map -> combine_with_previous
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => false,
        })
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let current = match &item.current.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            };
            let previous = item.previous.map(|prev| match &prev.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            });
            StreamItem::Value(format!("Current: {}, Previous: {:?}", current, previous))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Bob, Previous: None"
    );

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    tx.send(Sequenced::new(person_dave()))?; // 28 - filtered
    tx.send(Sequenced::new(person_diane()))?; // 40 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Diane, Previous: Some(\"Charlie\")"
    );

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_in_middle_of_chain_map_ordered() -> anyhow::Result<()> {
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: combine ages
    let age_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<u32> {
        let primary_age = match &state.values()[0] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let secondary_age = match &state.values()[1] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        TestWrapper::new(primary_age + secondary_age, state.timestamp())
    };

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), age_combiner)
        .map_ordered(|stream_item| async move {
            let age_sum = stream_item.clone().into_inner();
            StreamItem::Value(format!("Combined age: {}", age_sum))
        });

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(person_bob()))?; // 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await,
        StreamItem::Value("Combined age: 55".to_string())
    ); // 30 + 25

    primary_tx.send(Sequenced::new(person_charlie()))?; // 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await,
        StreamItem::Value("Combined age: 60".to_string())
    ); // 35 + 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    primary_tx.send(Sequenced::new(person_dave()))?; // 28

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await,
        StreamItem::Value("Combined age: 68".to_string())
    ); // 28 + 40

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain merge_with with map_ordered that doubles the counter
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 2)
        });

    // Send first value
    tx.send(Sequenced::new(person_alice()))?;

    // Assert first result: state=1, doubled=2
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        2,
        "First emission: (0+1)*2 = 2"
    );

    // Send second value
    tx.send(Sequenced::new(person_bob()))?;

    // Assert second result: state=2, doubled=4
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        4,
        "Second emission: (1+1)*2 = 4"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain merge_with with map, filter, and another map
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 3)
        })
        .filter_ordered(|&value| value > 6)
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value + 10)
        });

    // Send first value - state: 1, *3=3 (filtered out: 3 <= 6)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state: 2, *3=6 (filtered out: 6 <= 6)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state: 3, *3=9, +10=19 (kept: 9 > 6)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: first kept value
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        19,
        "Third emission: 3*3=9, 9+10=19"
    );

    // Send fourth value - state: 4, *3=12, +10=22 (kept: 12 > 6)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: second kept value
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        22,
        "Fourth emission: 4*3=12, 12+10=22"
    );

    Ok(())
}
